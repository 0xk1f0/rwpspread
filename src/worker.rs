use crate::cli::{Alignment, Backend, Config, Locker};
use crate::integrations::{
    helpers, hyprlock::Hyprlock, hyprpaper::Hyprpaper, palette::Palette, swaybg::Swaybg,
    swaylock::Swaylock, wpaperd::Wpaperd,
};
use crate::wayland::{Direction, Monitor};
use glob::glob;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use rand::seq::SliceRandom;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::cmp;
use std::collections::hash_map;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::os::unix;
use std::path::{Path, PathBuf};

pub struct ResultPaper {
    pub monitor_name: String,
    pub full_path: String,
}

pub struct Worker {
    hash: String,
    monitors: Vec<Monitor>,
    workdir: String,
    output: Vec<ResultPaper>,
}

impl Worker {
    pub fn new() -> Self {
        Self {
            hash: String::new(),
            monitors: Vec::new(),
            workdir: String::new(),
            output: Vec::new(),
        }
    }
    // split main image into two seperate, utilizes scaling
    pub fn run(&mut self, config: &Config, input_monitors: Vec<Monitor>) -> Result<(), String> {
        // pre run script check
        if let Some(pre_script_path) = &config.pre_path {
            helpers::run_oneshot(pre_script_path)?;
        }

        // check input image type
        let target_image: PathBuf;
        // unwrap is safe here since we checked the path previously
        if fs::metadata(&config.input_path).unwrap().is_dir() {
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
            self.workdir = format!("{}/.cache/rwpspread", env::var("HOME").unwrap());
            self.ensure_save_location(&self.workdir)?;
        } else {
            self.workdir = env::var("PWD").unwrap();
        }

        // calculate hash
        let mut hasher = hash_map::DefaultHasher::new();
        img.as_bytes().hash(&mut hasher);
        config.hash(&mut hasher);
        input_monitors.hash(&mut hasher);
        self.hash = format!("{:x}", hasher.finish());

        // set monitors
        if let Some(pixels) = config.compensate {
            self.monitors = self.bezel_compensate(input_monitors, pixels as i32)?;
        } else {
            self.monitors = input_monitors;
        }

        // check caches first
        let caches_present: bool = self.check_caches(&config);

        // do we need to resplit
        if config.force_resplit || !caches_present {
            // cleanup caches first
            self.cleanup_cache()?;

            // we need to resplit
            self.output = self.perform_split(img, config, &self.workdir)?;
        }

        // check if we need to handle a backend
        if let Some(backend) = &config.backend {
            // recheck what integration we're working with
            match backend {
                Backend::Wpaperd => {
                    // set and ensure config location
                    let config_location = format!("{}/.config/wpaperd", env::var("HOME").unwrap());
                    self.ensure_save_location(&config_location)?;

                    // create new wpaperd instance
                    let wpaperd = Wpaperd::new(
                        target_image.to_string_lossy().to_string(),
                        self.hash.clone(),
                        config_location + "/config.toml",
                    )?;

                    // check wpaper config hash
                    let wpaperd_present = wpaperd.check_existing();

                    // do we need to rebuild config
                    // also always rebuild when force resplit was set
                    if config.force_resplit || !wpaperd_present {
                        // yes we do
                        wpaperd.build(&self.output)?;
                        // restart
                        helpers::force_restart("wpaperd", vec![])?;
                    } else {
                        // only start if we're not running already
                        helpers::soft_restart("wpaperd", vec![])?;
                    }
                }
                Backend::Swaybg => {
                    // start or restart the swaybg instance
                    // considering present caches
                    if config.force_resplit || !caches_present {
                        let swaybg_args = Swaybg::new(&self.output)?;
                        helpers::force_restart("swaybg", swaybg_args)?;
                    } else {
                        // since swaybg has no config file, we need to assemble the names manually
                        for monitor in &self.monitors {
                            self.output.push(ResultPaper {
                                monitor_name: monitor.name.clone(),
                                full_path: format!(
                                    "{}/rwps_{}_{}.png",
                                    &self.workdir, &self.hash, monitor.name
                                ),
                            })
                        }
                        let swaybg_args = Swaybg::new(&self.output)?;
                        helpers::soft_restart("swaybg", swaybg_args)?;
                    }
                }
                Backend::Hyprpaper => {
                    // first soft restart
                    helpers::soft_restart("hyprpaper", vec![])?;
                    if config.force_resplit || !caches_present {
                        Hyprpaper::push(&self.output)?;
                    } else {
                        // hyprpaper also loads dynamically, so we need to manually assemble
                        for monitor in &self.monitors {
                            self.output.push(ResultPaper {
                                monitor_name: monitor.name.clone(),
                                full_path: format!(
                                    "{}/rwps_{}_{}.png",
                                    &self.workdir, &self.hash, monitor.name
                                ),
                            })
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
                        Hyprlock::new(&self.output, &self.workdir)?;
                    }
                }
                Locker::Swaylock => {
                    if !caches_present || config.force_resplit {
                        Swaylock::new(&self.output, &self.workdir)?;
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
            helpers::run_oneshot(post_script_path)?;
        }

        Ok(())
    }
    // compensate for bezels in pixel amount
    fn bezel_compensate(
        &self,
        mut input_monitors: Vec<Monitor>,
        shift_amount: i32,
    ) -> Result<Vec<Monitor>, String> {
        // check for touching displays
        let mut some_touching: bool = true;

        // iterate while we have something left to adjust
        while some_touching {
            some_touching = false;
            // create a copy to use as lookup
            let lookup_monitors = input_monitors.clone();
            input_monitors.iter_mut().any(|monitor| {
                lookup_monitors
                    .iter()
                    .find(|&node| {
                        if let Some(colission) = monitor.collides(node) {
                            some_touching = true;
                            match colission {
                                Direction::Up => {
                                    if !monitor.collides_at(&Direction::Down, node) {
                                        monitor.y += shift_amount;
                                    }
                                }
                                Direction::Down => {
                                    if !monitor.collides_at(&Direction::Up, node) {
                                        monitor.y -= shift_amount;
                                    }
                                }
                                Direction::Left => {
                                    if !monitor.collides_at(&Direction::Right, node) {
                                        monitor.x += shift_amount;
                                    }
                                }
                                Direction::Right => {
                                    if !monitor.collides_at(&Direction::Left, node) {
                                        monitor.x -= shift_amount;
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

        Ok(input_monitors)
    }
    // do the actual splitting
    fn perform_split(
        &self,
        mut input_image: DynamicImage,
        config: &Config,
        output_path: &String,
    ) -> Result<Vec<ResultPaper>, String> {
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
        for monitor in &self.monitors {
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
            // we can actually try to align the monitor layout since we have
            // some room to work with
            // assuming image is bigger than monitor layout
            match config.align.as_ref().unwrap() {
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
                    resize_offset_y = (input_image.dimensions().1 - (max_y + max_negative_y)) / 2;
                }
                Alignment::Lc => {
                    resize_offset_x = 0;
                    resize_offset_y = (input_image.dimensions().1 - (max_y + max_negative_y)) / 2;
                }
                Alignment::Ct => {
                    resize_offset_x = (input_image.dimensions().0 - (max_x + max_negative_x)) / 2;
                    resize_offset_y = (input_image.dimensions().1 - (max_y + max_negative_y)) / 2;
                }
            }
        }

        /*
            Crop image for screens
            and push them to the result vector, taking into
            account negative offsets
            Doing it in parallel using rayon for speedup
        */
        let crop_results: Vec<Result<ResultPaper, String>> = self
            .monitors
            .par_iter()
            .map(|monitor| -> Result<ResultPaper, String> {
                let adjusted_x = u32::try_from(origin_x as i32 + monitor.x)
                    .map_err(|_| "x adjustment out of range")?;
                let adjusted_y = u32::try_from(origin_y as i32 + monitor.y)
                    .map_err(|_| "y adjustment out of range")?;

                // crop to size
                let cropped_image = input_image.crop_imm(
                    adjusted_x + resize_offset_x,
                    adjusted_y + resize_offset_y,
                    monitor.width,
                    monitor.height,
                );

                // export to file
                let path_image =
                    format!("{}/rwps_{}_{}.png", output_path, &self.hash, &monitor.name);
                cropped_image
                    .save(&path_image)
                    .map_err(|err| err.to_string())?;
                // make a friendly name symlink to it
                // only if in daemon mode or with backend
                if config.daemon || config.backend.is_some() {
                    unix::fs::symlink(
                        &path_image,
                        format!("{}/rwps_{}.png", output_path, &monitor.name),
                    )
                    .map_err(|err| err.to_string())?;
                }

                Ok(ResultPaper {
                    monitor_name: format!("{}", &monitor.name),
                    full_path: path_image,
                })
            })
            .collect();

        // iterate and filter out the Ok() values
        let output_papers: Vec<ResultPaper> = crop_results
            .into_iter()
            .take_while(|entry| entry.is_ok())
            .filter_map(|result| result.ok())
            .collect();

        // if final papers length matches monitor count, we have no error
        if output_papers.len() == self.monitors.len() {
            Ok(output_papers)
        } else {
            Err("splitting error".to_string())
        }
    }

    fn select_random_image(&self, path: &PathBuf) -> Result<PathBuf, String> {
        // iterate over valid filetypes and push to vec
        let mut paths: Vec<PathBuf> = Vec::new();
        for ext in &["png", "jpg", "jpeg"] {
            let pattern = format!("{}/*.{}", path.to_string_lossy(), ext);
            for entry in glob(&pattern).expect("Failed to read glob pattern") {
                if let Ok(path) = entry {
                    paths.push(path);
                }
            }
        }
        // check if we found something
        if paths.is_empty() {
            return Err("Images directory empty".to_string());
        }
        // return random
        Ok(paths.choose(&mut rand::thread_rng()).unwrap().to_path_buf())
    }

    fn ensure_save_location(&self, path: &str) -> Result<(), String> {
        // try to create, notify if fail
        fs::create_dir_all(path).map_err(|_| "Failed to create Cache Directory")?;
        // else we're good
        Ok(())
    }

    fn cleanup_cache(&self) -> Result<(), String> {
        // wildcard search for our
        // images and delete them
        for entry in
            glob(&format!("{}/rwps_*", &self.workdir)).map_err(|_| "Failed to iterate directory")?
        {
            if let Ok(path) = entry {
                // yeet any file that we cached
                fs::remove_file(path).unwrap();
            }
        }

        Ok(())
    }

    fn check_caches(&self, config: &Config) -> bool {
        // what we search for
        let base_format = format!("{}/rwps_", &self.workdir);

        // path vector
        let mut path_list: Vec<(bool, String)> = Vec::new();

        // check for every monitor
        for monitor in &self.monitors {
            path_list.push((
                true,
                format!("{}{}_{}.png", base_format, &self.hash, monitor.name),
            ));
        }

        if let Some(locker) = &config.locker {
            path_list.push((true, format!("{}{}.conf", locker, base_format)));
        }
        path_list.push((config.palette, format!("{}colors.json", base_format)));

        // check if cache exists
        for path in path_list {
            if path.0 && !Path::new(&path.1).exists() {
                // we're missing something, regenerate
                return false;
            }
        }

        true
    }
}
