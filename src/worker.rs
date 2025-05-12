use crate::cli::{Alignment, Backend, Config, Locker};
use crate::helpers::Helpers;
use crate::integrations::{
    hyprlock::Hyprlock, hyprpaper::Hyprpaper, palette::Palette, swaybg::Swaybg, swaylock::Swaylock,
    wpaperd::Wpaperd,
};
use crate::wayland::{Direction, Monitor};
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
    monitors: HashMap<String, Monitor>,
    output: HashMap<String, String>,
}

impl Worker {
    pub fn new() -> Self {
        Self {
            hash: String::new(),
            workdir: String::new(),
            monitors: HashMap::new(),
            output: HashMap::new(),
        }
    }
    /// Initialize and run a new Worker instance
    pub fn run(
        &mut self,
        config: &Config,
        input_monitors: HashMap<String, Monitor>,
    ) -> Result<(), String> {
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

        // set monitors
        self.monitors = input_monitors;

        // ppi compensate if set
        if config.ppi {
            // check if all specified monitors exist
            if !config
                .monitors
                .iter()
                .all(|a| self.monitors.contains_key(a.0))
            {
                return Err("non-existent monitors specified!".to_string());
            };
            self.ppi_compensate(config);
        }

        // bezel compensate if set
        if let Some(pixels) = config.bezel {
            self.bezel_compensate(pixels as i32);
        }

        // calculate hash
        self.hash = self.calculate_blake3_hash(vec![
            serde::encode_to_vec(&config, config::standard())
                .map_err(|_| "serialization error".to_string())?
                .as_slice(),
            serde::encode_to_vec(&self.monitors, config::standard())
                .map_err(|_| "serialization error".to_string())?
                .as_slice(),
        ]);

        // check caches first
        let caches_present: bool = self.check_caches(&config).map_err(|err| err.to_string())?;

        // do we need to resplit
        if config.force_resplit || !caches_present {
            // cleanup caches first
            self.cleanup_cache()?;

            // we need to resplit
            let raw = self.perform_split(img, config)?;

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
                        for (name, _) in &self.monitors {
                            self.output.insert(
                                name.clone(),
                                format!("{}/rwps_{}_{}.png", &self.workdir, &self.hash, name),
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
                        for monitor in &self.monitors {
                            self.output.insert(
                                monitor.0.clone(),
                                format!("{}/rwps_{}_{}.png", &self.workdir, &self.hash, monitor.0),
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
        mut input_image: DynamicImage,
        config: &Config,
    ) -> Result<Arc<Mutex<HashMap<String, DynamicImage>>>, String> {
        /*
            Calculate Overall Size
            We can say that max width will be the biggest monitor
            with the greatest x-offset, max height will be defined in the same
            way except using y-offset.
            For Negative offsets, this will be the same except that we keep track
            of them in two seperate max variables.
            We also check how far negatively offset the biggest screen is so we can
            take it as the new origin.
        */
        let (mut max_x, mut max_y, mut max_negative_x, mut max_negative_y) = (0, 0, 0, 0);
        let (mut origin_x, mut origin_y) = (0, 0);
        for (_, monitor) in &self.monitors {
            // convert the negative values to positive ones
            let (abs_x, abs_y) = (monitor.x.abs() as u32, monitor.y.abs() as u32);
            // compare to max vals depending if positive or negative
            // also keep track of max negative offset
            // should offset be smaller than mon size, add back to positive
            if monitor.x.is_negative() {
                max_negative_x = cmp::max(abs_x, max_negative_x);
                origin_x = cmp::max(abs_x, origin_x);
                if abs_x < monitor.width {
                    max_x = cmp::max(monitor.width - abs_x, max_x);
                }
            } else {
                max_x = cmp::max(abs_x + monitor.width, max_x);
            }
            if monitor.y.is_negative() {
                max_negative_y = cmp::max(abs_y, max_negative_y);
                origin_y = cmp::max(abs_y, origin_y);
                if abs_y < monitor.height {
                    max_y = cmp::max(monitor.height - abs_y, max_y);
                }
            } else {
                max_y = cmp::max(abs_y + monitor.height, max_y);
            }
        }

        /*
            Check how we resize
            Either at users choice or if image is too small
            Should input be big enough, we can consider centering
        */
        let (mut resize_offset_x, mut resize_offset_y) = (0, 0);
        if config.align.is_none()
            || input_image.dimensions().0 < max_x + max_negative_x
            || input_image.dimensions().1 < max_y + max_negative_y
        {
            // scale image to fit calculated size
            input_image = input_image.resize_to_fill(
                max_x + max_negative_x,
                max_y + max_negative_y,
                FilterType::Lanczos3,
            );
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
                        resize_offset_y = input_image.dimensions().1 - (max_y + max_negative_y);
                    }
                    Alignment::Tr => {
                        resize_offset_x = input_image.dimensions().0 - (max_x + max_negative_x);
                        resize_offset_y = 0;
                    }
                    Alignment::Br => {
                        resize_offset_x = input_image.dimensions().0 - (max_x + max_negative_x);
                        resize_offset_y = input_image.dimensions().1 - (max_y + max_negative_y);
                    }
                    Alignment::Tc => {
                        resize_offset_x = input_image.dimensions().0 - (max_x + max_negative_x) / 2;
                        resize_offset_y = 0;
                    }
                    Alignment::Bc => {
                        resize_offset_x = input_image.dimensions().0 - (max_x + max_negative_x) / 2;
                        resize_offset_y = input_image.dimensions().1 - (max_y + max_negative_y);
                    }
                    Alignment::Rc => {
                        resize_offset_x = input_image.dimensions().0 - (max_x + max_negative_x);
                        resize_offset_y =
                            (input_image.dimensions().1 - (max_y + max_negative_y)) / 2;
                    }
                    Alignment::Lc => {
                        resize_offset_x = 0;
                        resize_offset_y =
                            (input_image.dimensions().1 - (max_y + max_negative_y)) / 2;
                    }
                    Alignment::Ct => {
                        resize_offset_x =
                            (input_image.dimensions().0 - (max_x + max_negative_x)) / 2;
                        resize_offset_y =
                            (input_image.dimensions().1 - (max_y + max_negative_y)) / 2;
                    }
                }
            }
        }

        /*
            Crop image for screens
            and push them to the result vector, taking into
            account negative offsets
            Doing it in parallel using rayon for speedup
        */
        let output: Arc<Mutex<HashMap<String, DynamicImage>>> =
            Arc::new(Mutex::new(HashMap::with_capacity(self.monitors.len())));
        self.monitors.par_iter().for_each(|monitor| {
            let adjusted_x = u32::try_from(origin_x as i32 + monitor.1.x);
            let adjusted_y = u32::try_from(origin_y as i32 + monitor.1.y);
            if let Ok(x) = adjusted_x {
                if let Ok(y) = adjusted_y {
                    // crop to size
                    output.lock().unwrap().insert(
                        monitor.0.clone(),
                        input_image
                            .crop_imm(
                                x + resize_offset_x,
                                y + resize_offset_y,
                                monitor.1.width,
                                monitor.1.height,
                            )
                            .resize_to_fill(
                                monitor.1.initial_width,
                                monitor.1.initial_height,
                                FilterType::Lanczos3,
                            ),
                    );
                }
            }
        });

        if output
            .try_lock()
            .map_err(|_| "could not aquire lock on split images")?
            .len()
            == self.monitors.len()
        {
            Ok(output)
        } else {
            Err("initial splitting error".to_string())
        }
    }
    /// Compensate for bezels in pixel amount
    fn bezel_compensate(&mut self, shift_amount: i32) {
        // iterate while we have something left to adjust
        let mut has_collision: bool = true;
        while has_collision {
            // create a copy to use as lookup
            let lookup = self.monitors.clone();
            has_collision = self.monitors.values_mut().any(|monitor| {
                lookup
                    .values()
                    .find(|&neighbor| {
                        if let Some(collision) = monitor.collides_with(neighbor) {
                            match collision {
                                Direction::Up => {
                                    if !monitor.collides_with_at(neighbor, &Direction::Down) {
                                        monitor.shift(0, shift_amount);
                                    }
                                }
                                Direction::Down => {
                                    if !monitor.collides_with_at(neighbor, &Direction::Up) {
                                        monitor.shift(0, -shift_amount);
                                    }
                                }
                                Direction::Left => {
                                    if !monitor.collides_with_at(neighbor, &Direction::Right) {
                                        monitor.shift(shift_amount, 0);
                                    }
                                }
                                Direction::Right => {
                                    if !monitor.collides_with_at(neighbor, &Direction::Left) {
                                        monitor.shift(-shift_amount, 0);
                                    }
                                }
                            }
                            true
                        } else {
                            false
                        }
                    })
                    .is_some()
            });
        }
    }
    /// Compensate for different monitor ppi values
    fn ppi_compensate(&mut self, config: &Config) {
        let ppi_min = self
            .monitors
            .par_iter()
            .map(|monitor| {
                if let Some(&diagonal_inches) = config.monitors.get(monitor.0) {
                    monitor.1.ppi(diagonal_inches)
                } else {
                    0
                }
            })
            .min();

        if let Some(reference_ppi) = ppi_min {
            config
                .monitors
                .iter()
                .for_each(|(monitor_name, &diagonal_inches)| {
                    let lookup = self.monitors.clone();
                    if let Some(monitor) = lookup.get(monitor_name) {
                        let factor = reference_ppi as f32 / monitor.ppi(diagonal_inches) as f32;
                        let mut neighbors: HashMap<&String, &Direction> = HashMap::new();
                        lookup.iter().for_each(|neighbor| {
                            if let Some(collision) = monitor.collides_with(neighbor.1) {
                                // insert the name of the neighboring monitor and
                                // the collision direction of the scaled monitor
                                neighbors.insert(monitor_name, collision);
                            }
                        });
                        if let Some(monitor) = self.monitors.get_mut(monitor_name) {
                            monitor.scale(factor).center();
                        }
                        neighbors
                            .iter()
                            .for_each(|(_, &direction)| match direction {
                                Direction::Up => {
                                    if !neighbors.values().any(|&val| *val == Direction::Down) {
                                        if let Some(monitor) = self.monitors.get_mut(monitor_name) {
                                            monitor.shift(0, -monitor.diff_shifts().1.abs());
                                        }
                                    }
                                }
                                Direction::Down => {
                                    if !neighbors.values().any(|&val| *val == Direction::Up) {
                                        if let Some(monitor) = self.monitors.get_mut(monitor_name) {
                                            monitor.shift(0, monitor.diff_shifts().1.abs());
                                        }
                                    }
                                }
                                Direction::Left => {
                                    if !neighbors.values().any(|&val| *val == Direction::Right) {
                                        if let Some(monitor) = self.monitors.get_mut(monitor_name) {
                                            monitor.shift(-monitor.diff_shifts().0.abs(), 0);
                                        }
                                    }
                                }
                                Direction::Right => {}
                            });
                    }
                });
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
    fn check_caches(&self, config: &Config) -> Result<bool, String> {
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
        for (name, _) in &self.monitors {
            runtime_paths.push(format!(
                "{}/rwps_{}_{}.png",
                &self.workdir, name, &self.hash
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
