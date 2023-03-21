use std::cmp;
use std::env::var;
use image::{GenericImageView, DynamicImage, imageops::FilterType};
use md5::{compute, Digest};
use glob::glob;
use crate::Config;
use crate::wpaperd::{WpaperdConfig, cmd_wrapper};
use crate::layout::Monitor;

pub struct ResultPaper {
    pub monitor_name: String,
    pub image_full_path: String,
    pub image: DynamicImage,
}

pub struct Splitter {
    base_hash: String,
    save_path: String,
    monitors: Vec<Monitor>,
    result_papers: Vec<ResultPaper>,
}

impl Splitter {
    pub fn new() -> Self {
        Self {
            base_hash: String::new(),
            save_path: format!(
                "{}/.cache/",
                var("HOME").unwrap()
            ),
            monitors: Vec::new(),
            result_papers: Vec::new()
        }
    }

    // split main image into two seperate, utilizes scaling
    pub fn run(mut self, config: &Config) -> Result<(), String> {
        // open original input image
        let mut img = image::open(
            &config.image_path
        ).map_err(
            |err| err.to_string()
        )?;

        // fetch monitors
        self.monitors = Monitor::new_from_hyprland().map_err(
            |err| err.to_string()
        )?;

        // calculate hash
        self.base_hash = self.hash_config(
            compute(img.as_bytes()),
            config
        );

        // check caches and config force bool
        if 
            self.check_caches() &&
            ! config.force_resplit 
        {
            // caches exist
            // run wrapper
            cmd_wrapper().map_err(
                |err| err.to_string()
            )?;

            // exit since wrapper will be run
            return Ok(())
        }

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
            overall_width = cmp::max(
                monitor.width + monitor.x as u32,
                overall_width
            );
            overall_height = cmp::max(
                monitor.height + monitor.y as u32,
                overall_height
            );
        }

        // check if we need to resize
        // either if user doesn't deny
        // or if image is too small
        if 
            config.dont_downscale == false
            || img.dimensions().0 < overall_width
            || img.dimensions().1 < overall_height
        {
            // scale image to fit calculated size
            img = img.resize_to_fill(
                overall_width,
                overall_height,
                FilterType::Lanczos3
            );
        }

        // Crop image for screens
        for monitor in &self.monitors {
            let cropped_image = img.crop(
                monitor.x as u32,
                monitor.y as u32,
                monitor.width,
                monitor.height
            );
            self.result_papers.push(
                ResultPaper { 
                    monitor_name: format!("{}", &monitor.name),
                    image_full_path: format!(
                        "{}rwps_{}_{}.png",
                        &self.save_path,
                        &self.base_hash[2..12],
                        format!("{}", &monitor.name),
                    ),
                    image: cropped_image
                }
            )
        }

        // save our result images
        for paper in &self.result_papers {
            paper.image.save(
                &paper.image_full_path
            ).map_err(
                |err| err.to_string()
            )?;
        }

        // check if we need to generate wpaperd config
        if config.with_wpaperd {
            // create new wpaperd instance
            let wpaperd = WpaperdConfig::new(
                format!(
                    "{}/.config/wpaperd/output.conf",
                    var("HOME").unwrap()
                ),
                self.base_hash
            );

            //check wpaper config hash
            if
                config.with_wpaperd && 
                ! config.force_resplit 
            {
                if let true = wpaperd.check_existing().map_err(
                    |err| err.to_string()
                )? {
                    // match, don't rebuild
                }
                else {
                    // we need to rebuild
                    wpaperd.build(&self.result_papers).map_err(
                        |err| err.to_string()
                    )?;
                }
            }

            // finally, run wrapper
            cmd_wrapper().map_err(
                |err| err.to_string()
            )?;
        }

        // return
        Ok(())
    }

    fn hash_config(&self, img_hash: Digest, cfg: &Config) -> String {
        // new hash string
        let mut hash_string = String::new();

        // loop over config params and add to string
        for monitor in &self.monitors {
            hash_string.push_str(&monitor.to_string());
        }

        // compute and assemble hash
        // we also factor in downscaling as images
        // might be different if we dont downscale
        format!(
            "# {:?}{:?}{:?}\n",
            img_hash,
            compute(cfg.dont_downscale.to_string()),
            compute(hash_string.as_bytes())
        )
    }

    fn check_caches(&self) -> bool {
        // wildacrd search for cached images
        for entry in glob(
            &format!(
                "{}/.cache/rwps_{}*",
                var("HOME").unwrap(),
                &self.base_hash[2..12]
            )
        ).unwrap() {
            match entry {
                Ok(_) => return true,
                Err(_) => break
            }
        }

        // if we dont exit sooner, return false
        false
    }
}
