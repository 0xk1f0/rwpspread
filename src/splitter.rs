use crate::palette::Palette;
use crate::wayland::Monitor;
use crate::wpaperd::{CmdWrapper, WpaperdConfig};
use crate::Config;
use glob::glob;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use std::cmp;
use std::collections::hash_map::DefaultHasher;
use std::env::var;
use std::fs::remove_file;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub struct ResultPaper {
    pub monitor_name: String,
    pub full_path: String,
    pub image: DynamicImage,
}

pub struct Splitter {
    hash: String,
    monitors: Vec<Monitor>,
    result_papers: Vec<ResultPaper>,
}

impl Splitter {
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
        let mut hasher = DefaultHasher::new();
        img.as_bytes().hash(&mut hasher);
        config.hash(&mut hasher);
        self.monitors.hash(&mut hasher);
        self.hash = format!("{:x}", hasher.finish());

        // check for palette bool and do that first
        if config.with_palette {
            let color_palette = Palette::new(&config.image_path).map_err(|err| err.to_string())?;
            color_palette
                .generate_mostused(format!("{}/.cache/rwpcolors.json", var("HOME").unwrap()))
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
                    .perform_split(img, config, format!("{}/.cache", var("HOME").unwrap()))
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
                .perform_split(img, config, var("PWD").unwrap())
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
            @TODO: Might need Changes.
            Calculate Overall Size
            We can say that max width will be the biggest monitor
            with the greatest x-offset, max height will be defined in the same
            way except using y-offset.
            Taking account of negative offset, which might be possible.
            In theory biggest size will still be same as above, except that we
            need to make the negative offsets always positive, to ge the total span.
        */
        let (mut overall_width, mut overall_height) = (0, 0);
        for monitor in &self.monitors {
            let mut adjusted_x: i32 = monitor.x;
            let mut adjusted_y: i32  = monitor.y;
            // special case for negative offsets
            if monitor.x < 0 {
                adjusted_x = monitor.x * -1;
            } else if monitor.y < 0 {
                adjusted_y = monitor.y * -1;
            }
            overall_width = cmp::max(monitor.width + adjusted_x as u32, overall_width);
            overall_height = cmp::max(monitor.height + adjusted_y as u32, overall_height);
        }

        /*
            Check how we resize
            Either at users choice or if image is too small
            Should input be big enough, we can consider centering
        */
        let (mut resize_offset_x, mut resize_offset_y) = (0, 0);
        if config.center == false
            || img.dimensions().0 < overall_width
            || img.dimensions().1 < overall_height
        {
            // scale image to fit calculated size
            img = img.resize_to_fill(overall_width, overall_height, FilterType::Lanczos3);
        } else {
            // we can actually try to center the monitor layout since we have
            // some room to work with
            // assuming image is bigger than monitor layout
            resize_offset_x = (img.dimensions().0 - overall_width) / 2;
            resize_offset_y = (img.dimensions().1 - overall_height) / 2;
        }

        /*
            @TODO: Might need Changes, same as above.
            Crop image for screens
            and push them to the result vector, taking into
            account negative offsets
        */
        let mut result = Vec::with_capacity(self.monitors.len());
        for monitor in &self.monitors {
            let mut adjusted_x: i32 = monitor.x;
            let mut adjusted_y: i32  = monitor.y;
            // special case for negative offsets
            if monitor.x < 0 {
                adjusted_x = monitor.x * -1;
            } else if monitor.y < 0 {
                adjusted_y = monitor.y * -1;
            }
            // crop the image
            let cropped_image = img.crop(
                adjusted_x as u32 + resize_offset_x,
                adjusted_y as u32 + resize_offset_y,
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
        for entry in glob(&format!("{}/.cache/rwps_*", var("HOME").unwrap())).unwrap() {
            if let Ok(path) = entry {
                // yeet any file that we cached
                remove_file(path).unwrap();
            }
        }
    }

    fn check_caches(&self) -> bool {
        // what we search for
        let base_format = format!("{}/.cache/rwps_{}", var("HOME").unwrap(), &self.hash);

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
