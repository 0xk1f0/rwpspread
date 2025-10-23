use crate::cli::{Alignment, Backend, Config, Locker};
use crate::helpers::Helpers;
use crate::integrations::{
    hyprlock::Hyprlock, hyprpaper::Hyprpaper, palette::Palette, swaybg::Swaybg, swaylock::Swaylock,
    wpaperd::Wpaperd,
};
use crate::layout::{Layout, LayoutMonitor};
use crate::wayland::Monitor;
use bincode::{config, serde};
use glob::glob;
use image::{DynamicImage, GenericImageView, imageops::FilterType};
use rand::seq::IndexedRandom;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::cmp;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::os::unix;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct Worker {
    hash: String,
    workdir: String,
    output: HashMap<String, String>,
}

impl Worker {
    pub fn new() -> Self {
        Self {
            hash: String::new(),
            workdir: String::new(),
            output: HashMap::new(),
        }
    }
    /// Initialize and run a new Worker instance
    pub fn run(&mut self, config: &Config, monitors: Vec<Monitor>) -> Result<(), String> {
        // pre run script check
        if let Some(pre_script_path) = &config.pre_path {
            Helpers::run_oneshot(pre_script_path)?;
        }

        // check input image type
        let target_image: PathBuf;
        if fs::metadata(&config.input_path)
            .map_err(|err| err.to_string())?
            .is_dir()
        {
            // image is random from directory
            target_image = self.select_random_image(&config.input_path)?;
        } else {
            // image is actual input
            target_image = config.input_path.to_owned();
        }

        // open original input image
        let img = image::open(&target_image).map_err(|_| "failed to open image")?;

        // set workdir location
        if let Some(output_path) = &config.output_path {
            self.workdir = output_path.to_owned();
        } else if config.daemon || config.backend.is_some() {
            self.workdir = format!(
                "{}/.cache/rwpspread",
                env::var("HOME").map_err(|_| "failed read $HOME")?
            );
            self.ensure_path(&self.workdir)?;
        } else {
            self.workdir = env::var("PWD").map_err(|_| "failed read $PWD")?;
        }

        // ppi compensate if set
        if config.ppi {
            if !monitors
                .iter()
                .all(|a| config.monitors.contains_key(&a.name))
            {
                return Err("missing monitor definitions!".to_string());
            };
        }

        // calculate hash
        self.hash = self.calculate_blake3_hash(vec![
            serde::encode_to_vec(&config, config::standard())
                .map_err(|_| "serialization error".to_string())?
                .as_slice(),
            serde::encode_to_vec(&monitors, config::standard())
                .map_err(|_| "serialization error".to_string())?
                .as_slice(),
        ]);

        // check caches first
        let caches_present: bool = self
            .check_caches(&config, &monitors)
            .map_err(|err| err.to_string())?;

        // do we need to resplit
        if config.force_resplit || !caches_present {
            // cleanup caches first
            self.cleanup_cache()?;

            // we need to resplit
            let raw = self.perform_split(&monitors, img, config)?;

            // save to path
            self.output = self.export_images(config, raw, &self.workdir)?;
        }

        // check if we need to handle a backend
        if let Some(backend) = &config.backend {
            // recheck what integration we're working with
            match backend {
                Backend::Wpaperd => {
                    // set and ensure config location
                    let config_path = format!(
                        "{}/.config/wpaperd/config.toml",
                        env::var("HOME").map_err(|_| "failed read $HOME")?
                    );
                    self.ensure_path(&config_path)?;

                    // check wpaper config hash
                    let is_cached = Wpaperd::check_existing(&config_path, &self.hash);

                    // do we need to rebuild config
                    // also always rebuild when force resplit was set
                    if config.force_resplit || !is_cached {
                        // yes we do
                        Wpaperd::new(&config_path, &self.hash, &self.output)?;
                        // restart
                        Helpers::force_restart("wpaperd", vec![])?;
                    } else {
                        // only start if we're not running already
                        Helpers::soft_restart("wpaperd", vec![])?;
                    }
                }
                Backend::Swaybg => {
                    // start or restart the swaybg instance
                    // considering present caches
                    if config.force_resplit || !caches_present {
                        let swaybg_args = Swaybg::new(&self.output)?;
                        Helpers::force_restart("swaybg", swaybg_args)?;
                    } else {
                        // since swaybg has no config file, we need to assemble the names manually
                        for mon in monitors {
                            self.output.insert(
                                mon.name.to_owned(),
                                format!("{}/rwps_{}_{}.png", &self.workdir, &self.hash, mon.name),
                            );
                        }
                        let swaybg_args = Swaybg::new(&self.output)?;
                        Helpers::soft_restart("swaybg", swaybg_args)?;
                    }
                }
                Backend::Hyprpaper => {
                    // first soft restart
                    Helpers::soft_restart("hyprpaper", vec![])?;
                    if config.force_resplit || !caches_present {
                        Hyprpaper::push(&self.output)?;
                    } else {
                        // hyprpaper also loads dynamically, so we need to manually assemble
                        for monitor in monitors {
                            self.output.insert(
                                monitor.name.to_owned(),
                                format!(
                                    "{}/rwps_{}_{}.png",
                                    &self.workdir, &self.hash, monitor.name
                                ),
                            );
                        }
                        Hyprpaper::push(&self.output)?;
                    }
                }
            }
        }

        // check if we need to generate a locker config
        if let Some(locker) = &config.locker {
            match locker {
                Locker::Hyprlock => {
                    if !caches_present || config.force_resplit {
                        Hyprlock::new(&self.workdir, &self.output)?;
                    }
                }
                Locker::Swaylock => {
                    if !caches_present || config.force_resplit {
                        Swaylock::new(&self.workdir, &self.output)?;
                    }
                }
            }
        }

        // check for palette bool
        if config.palette && !caches_present || config.force_resplit {
            let color_palette = Palette::new(&target_image)?;
            color_palette.generate(&self.workdir)?;
        }

        // post run script check
        if let Some(post_script_path) = &config.post_path {
            Helpers::run_oneshot(post_script_path)?;
        }

        Ok(())
    }
    /// Perform the main splitting logic and return the resulting split images
    fn perform_split(
        &self,
        monitors: &[Monitor],
        mut input_image: DynamicImage,
        config: &Config,
    ) -> Result<Arc<Mutex<HashMap<String, DynamicImage>>>, String> {
        let mut layout = Layout::from_monitors(monitors);

        if config.ppi {
            let mut diagonals: Vec<u32> = Vec::with_capacity(config.monitors.len());
            for monitor in &config.monitors {
                diagonals.push(monitor.1.to_owned())
            }
            layout.compensate_ppi(diagonals);
        }

        let bezel_amount;
        if let Some(amount) = config.bezel {
            bezel_amount = amount;
        } else {
            bezel_amount = 0;
        }

        // resolve layout
        layout.resolve_layout(bezel_amount, 100);

        // find max needed image size
        let (mut max_x, mut max_y) = (0, 0);
        for monitor in &layout.monitors {
            max_x = cmp::max(monitor.x1 + monitor.width as i32, max_x);
            max_y = cmp::max(monitor.y1 + monitor.height as i32, max_y);
        }

        // check if we can align the layout to a bigger input image
        let (mut resize_offset_x, mut resize_offset_y) = (0, 0);
        if config.align.is_none()
            || input_image.dimensions().0 < max_x as u32
            || input_image.dimensions().1 < max_y as u32
        {
            // scale image to fit calculated size
            input_image =
                input_image.resize_to_fill(max_x as u32, max_y as u32, FilterType::Lanczos3);
        } else {
            // we align the monitor layout since we have some room to work with
            if let Some(alignment) = &config.align {
                match alignment {
                    Alignment::Tl => {
                        resize_offset_x = 0;
                        resize_offset_y = 0;
                    }
                    Alignment::Bl => {
                        resize_offset_x = 0;
                        resize_offset_y = input_image.dimensions().1 - max_y as u32;
                    }
                    Alignment::Tr => {
                        resize_offset_x = input_image.dimensions().0 - max_x as u32;
                        resize_offset_y = 0;
                    }
                    Alignment::Br => {
                        resize_offset_x = input_image.dimensions().0 - max_x as u32;
                        resize_offset_y = input_image.dimensions().1 - max_y as u32;
                    }
                    Alignment::Tc => {
                        resize_offset_x = input_image.dimensions().0 - max_x as u32 / 2;
                        resize_offset_y = 0;
                    }
                    Alignment::Bc => {
                        resize_offset_x = input_image.dimensions().0 - max_x as u32 / 2;
                        resize_offset_y = input_image.dimensions().1 - max_y as u32;
                    }
                    Alignment::Rc => {
                        resize_offset_x = input_image.dimensions().0 - max_x as u32;
                        resize_offset_y = (input_image.dimensions().1 - max_y as u32) / 2;
                    }
                    Alignment::Lc => {
                        resize_offset_x = 0;
                        resize_offset_y = (input_image.dimensions().1 - max_y as u32) / 2;
                    }
                    Alignment::Ct => {
                        resize_offset_x = (input_image.dimensions().0 - max_x as u32) / 2;
                        resize_offset_y = (input_image.dimensions().1 - max_y as u32) / 2;
                    }
                }
            }
        }

        let mut output_monitors: HashMap<String, LayoutMonitor> = HashMap::new();
        for (modified, original) in layout.monitors.iter().zip(monitors) {
            output_monitors.insert(original.name.to_owned(), *modified);
        }

        let output: Arc<Mutex<HashMap<String, DynamicImage>>> =
            Arc::new(Mutex::new(HashMap::with_capacity(monitors.len())));
        output_monitors.par_iter().for_each(|monitor| {
            output.lock().unwrap().insert(
                monitor.0.to_owned(),
                input_image
                    .crop_imm(
                        monitor.1.x1 as u32 + resize_offset_x,
                        monitor.1.y1 as u32 + resize_offset_y,
                        monitor.1.width,
                        monitor.1.height,
                    )
                    .resize_to_fill(monitor.1.width, monitor.1.height, FilterType::Lanczos3),
            );
        });

        if output
            .try_lock()
            .map_err(|_| "could not aquire lock on split images")?
            .len()
            == monitors.len()
        {
            Ok(output)
        } else {
            Err("initial splitting error".to_string())
        }
    }
    /// Export and save the images on disk and return their paths
    fn export_images(
        &self,
        config: &Config,
        images: Arc<Mutex<HashMap<String, DynamicImage>>>,
        output_path: &String,
    ) -> Result<HashMap<String, String>, String> {
        images
            .try_lock()
            .map_err(|_| "could not aquire lock on split images")?
            .iter()
            .map(|image| {
                // export to file
                let path_image = format!("{}/rwps_{}_{}.png", output_path, image.0, &self.hash);
                image.1.save(&path_image).map_err(|err| err.to_string())?;
                // make a friendly name symlink to it
                // only if in daemon mode, backend or locker
                if config.daemon || config.backend.is_some() || config.locker.is_some() {
                    unix::fs::symlink(&path_image, format!("{}/rwps_{}.png", output_path, image.0))
                        .map_err(|err| err.to_string())?;
                }
                Ok((image.0.to_owned(), path_image))
            })
            .collect()
    }
    /// Calculate the blake3 hash of input vector
    fn calculate_blake3_hash(&self, input_items: Vec<&[u8]>) -> String {
        // create a new blake3 instance and hash all input items
        let mut hasher = blake3::Hasher::new();
        for item in input_items {
            hasher.update(item);
        }
        hasher.finalize().to_hex().as_str().to_owned()
    }
    /// Select and return a path to a random image in a folder
    fn select_random_image(&self, path: &PathBuf) -> Result<PathBuf, String> {
        // iterate over valid filetypes and push to vec
        let mut paths: Vec<PathBuf> = Vec::new();
        for ext in &["png", "jpg", "jpeg"] {
            let pattern = format!("{}/*.{}", path.display(), ext);
            for entry in glob(&pattern).expect("Failed to read glob pattern") {
                if let Ok(path) = entry {
                    paths.push(path);
                }
            }
        }
        // check if empty, else return
        if let Some(path) = paths.choose(&mut rand::rng()) {
            Ok(path.to_owned())
        } else {
            Err("Images directory empty".to_string())
        }
    }
    /// Ensure a path on disk exists
    fn ensure_path(&self, path: &str) -> Result<(), String> {
        fs::create_dir_all(
            &PathBuf::from(path)
                .parent()
                .ok_or("failed to determine path parent")?,
        )
        .map_err(|_| "failed to create directory path")?;

        Ok(())
    }
    /// Cleanup all cached items
    fn cleanup_cache(&self) -> Result<(), String> {
        // wildcard search for our images and delete them
        for entry in
            glob(&format!("{}/rwps_*", &self.workdir)).map_err(|_| "failed to iterate directory")?
        {
            if let Ok(path) = entry {
                fs::remove_file(path).map_err(|_| "failed to clear cache")?;
            }
        }

        Ok(())
    }
    /// Check if cached items exist and match current hash
    fn check_caches(&self, config: &Config, monitors: &Vec<Monitor>) -> Result<bool, String> {
        // find and assemble all paths with correct prefix
        let mut found_paths: Vec<String> = Vec::new();
        for path in glob(&format!("{}/rwps_*", &self.workdir))
            .map_err(|_| "failed to iterate directory")?
            .filter_map(Result::ok)
        {
            found_paths.push(path.display().to_string());
        }

        // assemble current runtime paths
        let mut runtime_paths: Vec<String> = Vec::new();
        for mon in monitors {
            runtime_paths.push(format!(
                "{}/rwps_{}_{}.png",
                &self.workdir, mon.name, &self.hash
            ));
        }
        if let Some(locker) = &config.locker {
            runtime_paths.push(format!("{}/rwps_{}.conf", &self.workdir, locker));
        }
        if config.palette {
            runtime_paths.push(format!("{}/rwps_colors.json", &self.workdir));
        }

        // serialize to hashable format
        let found_hash = serde::encode_to_vec(found_paths.as_slice(), config::standard())
            .map_err(|_| "serialization error".to_string())?;
        let runtime_hash = serde::encode_to_vec(runtime_paths.as_slice(), config::standard())
            .map_err(|_| "serialization error".to_string())?;
        // calculate hashes and return the compared result
        Ok(self.calculate_blake3_hash(vec![found_hash.as_slice()])
            == self.calculate_blake3_hash(vec![runtime_hash.as_slice()]))
    }
}
