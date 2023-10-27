use crate::palette::Palette;
use crate::swaylock::Swaylock;
use crate::wayland::Monitor;
use crate::wpaperd::{CmdWrapper, WpaperdConfig};
use crate::Config;
use glob::glob;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use std::cmp;
use std::collections::hash_map;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub struct ResultPaper {
    pub monitor_name: String,
    pub full_path: String,
    pub image: DynamicImage,
}

pub struct Worker {
    hash: String,
    monitors: Vec<Monitor>,
    result_papers: Vec<ResultPaper>,
}

impl Worker {
    pub fn new() -> Self {
        Self {
            hash: String::new(),
            monitors: Vec::new(),
            result_papers: Vec::new(),
        }
    }

    // split main image into two seperate, utilizes scaling
    pub fn run(&mut self, config: &Config, mon_vec: Vec<Monitor>) -> Result<(), String> {
        // open original input image
        let img = image::open(&config.image_path).map_err(|_| "failed to open image")?;

        // set monitors
        self.monitors = mon_vec;

        // calculate hash
        let mut hasher = hash_map::DefaultHasher::new();
        img.as_bytes().hash(&mut hasher);
        config.hash(&mut hasher);
        self.monitors.hash(&mut hasher);
        self.hash = format!("{:x}", hasher.finish());

        // check for palette bool and do that first
        if config.with_palette {
            let color_palette = Palette::new(&config.image_path).map_err(|err| err.to_string())?;
            color_palette
                .generate_mostused(format!("{}/.cache", env::var("HOME").unwrap()))
                .map_err(|err| err.to_string())?;
        }

        // check if we need to generate wpaperd config
        if config.with_wpaperd {
            // create new wpaperd instance
            let wpaperd = WpaperdConfig::new(
                config.image_path.to_string_lossy().to_string(),
                self.hash.clone(),
            );

            // check caches
            let caches_present = self.check_caches();

            // do we need to resplit
            if config.force_resplit || !caches_present {
                // cleanup caches first
                self.cleanup_cache();

                // we need to resplit
                self.result_papers = self
                    .perform_split(img, config, format!("{}/.cache", env::var("HOME").unwrap()))
                    .map_err(|err| err.to_string())?;
            }

            //check wpaper config hash
            let wpaperd_present = wpaperd.check_existing();

            // do we need to rebuild config
            // also always rebuild when force resplit was set
            if config.force_resplit || !wpaperd_present {
                // yes we do
                wpaperd
                    .build(&self.result_papers)
                    .map_err(|err| err.to_string())?;

                // restart
                CmdWrapper::restart().map_err(|err| err.to_string())?;
            }

            // only start if we're not running already
            CmdWrapper::soft_restart().map_err(|err| err.to_string())?;

        // no wpaperd to worry about, just split
        } else {
            // just split
            self.result_papers = self
                .perform_split(img, config, env::var("PWD").unwrap())
                .map_err(|err| err.to_string())?;
        }

        // check if we need to generate for swaylock
        if config.with_swaylock {
            Swaylock::generate(
                &self.result_papers,
                format!("{}/.cache", env::var("HOME").unwrap()),
            )
            .map_err(|err| err.to_string())?;
        }

        // return
        Ok(())
    }

    // do the actual splitting
    fn perform_split(
        &self,
        mut img: DynamicImage,
        config: &Config,
        save_path: String,
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
        if config.center == false
            || img.dimensions().0 < max_x + max_negative_x
            || img.dimensions().1 < max_y + max_negative_y
        {
            // scale image to fit calculated size
            img = img.resize_to_fill(
                max_x + max_negative_x,
                max_y + max_negative_y,
                FilterType::Lanczos3,
            );
        } else {
            // we can actually try to center the monitor layout since we have
            // some room to work with
            // assuming image is bigger than monitor layout
            resize_offset_x = (img.dimensions().0 - (max_x + max_negative_x)) / 2;
            resize_offset_y = (img.dimensions().1 - (max_y + max_negative_y)) / 2;
        }

        /*
            Crop image for screens
            and push them to the result vector, taking into
            account negative offsets
        */
        let mut result = Vec::with_capacity(self.monitors.len());
        for monitor in &self.monitors {
            // convert for cropping
            let adjusted_x = u32::try_from(origin_x as i32 + monitor.x)
                .map_err(|_| "x adjustment out of range")?;
            let adjusted_y = u32::try_from(origin_y as i32 + monitor.y)
                .map_err(|_| "y adjustment out of range")?;
            // crop the image
            let cropped_image = img.crop(
                adjusted_x + resize_offset_x,
                adjusted_y + resize_offset_y,
                monitor.width,
                monitor.height,
            );

            // get full image path
            let path_image = format!("{}/rwps_{}_{}.png", save_path, &self.hash, &monitor.name,);

            // save it
            cropped_image
                .save(&path_image)
                .map_err(|err| err.to_string())?;

            // push to result vector
            result.push(ResultPaper {
                monitor_name: format!("{}", &monitor.name),
                full_path: path_image,
                image: cropped_image,
            })
        }

        Ok(result)
    }

    fn cleanup_cache(&self) {
        // wildcard search for our
        // images and delete them
        for entry in glob(&format!("{}/.cache/rwps_*", env::var("HOME").unwrap())).unwrap() {
            if let Ok(path) = entry {
                // yeet any file that we cached
                fs::remove_file(path).unwrap();
            }
        }
    }

    fn check_caches(&self) -> bool {
        // what we search for
        let base_format = format!("{}/.cache/rwps_{}", env::var("HOME").unwrap(), &self.hash);

        // check for every monitor
        for monitor in &self.monitors {
            let image_path = format!("{}_{}.png", base_format, monitor.name);
            // check if a cached image exists
            if !Path::new(&image_path).exists() {
                // we're missing an image, regenerate
                return false;
            }
        }

        // if we pass, we're good
        true
    }
}