use crate::palette::Palette;
use crate::wayland::{Monitor, MonitorConfig};
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
    pub fn run(mut self, config: &Config) -> Result<(), String> {
        // open original input image
        let img = image::open(&config.image_path).map_err(|_| "failed to open image")?;

        // fetch monitors
        self.monitors = MonitorConfig::new().map_err(|err| err.to_string())?;

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
            let wpaperd = WpaperdConfig::new(self.hash.clone());

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
            Calculate Overall Size
            Assuming a monitor can never be negatively offset
            from root, we can say that max width will be the biggest monitor
            with the greatest x-offset, max height will be defined in the same
            way except using y-offset
        */
        let mut overall_width = 0;
        let mut overall_height = 0;
        for monitor in &self.monitors {
            overall_width = cmp::max(monitor.width + monitor.x as u32, overall_width);
            overall_height = cmp::max(monitor.height + monitor.y as u32, overall_height);
        }

        // check if we need to resize
        // either if user doesn't deny
        // or if image is too small
        if config.dont_downscale == false
            || img.dimensions().0 < overall_width
            || img.dimensions().1 < overall_height
        {
            // scale image to fit calculated size
            img = img.resize_to_fill(overall_width, overall_height, FilterType::Lanczos3);
        }

        // Crop image for screens
        // and push them to the result vector
        let mut result = Vec::with_capacity(self.monitors.len());
        for monitor in &self.monitors {
            // crop the image
            let cropped_image = img.crop(
                monitor.x as u32,
                monitor.y as u32,
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
