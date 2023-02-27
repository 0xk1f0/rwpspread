use image::{GenericImageView, DynamicImage, imageops::FilterType};
use std::cmp;
use std::env::var;
use std::path::PathBuf;
use md5::compute;
use colored::Colorize;
use crate::Config;
use crate::wpaperd::{WpaperdConfig, check_existing};
use crate::layout::Monitor;

// result paper struct
pub struct ResultPaper {
    pub monitor_name: String,
    pub image_full_path: String,
    pub image: DynamicImage,
}

pub struct Splitter {
    base_image_path: PathBuf,
    base_hash: String,
    save_path: String,
    monitors: Vec<Monitor>,
    result_papers: Vec<ResultPaper>,
    with_wpaperd: bool
}

impl Splitter {
    pub fn new(cfg: Config) -> Self {
        Self {
            base_image_path: cfg.image_path,
            base_hash: String::new(),
            save_path: format!("{}/.cache/", var("HOME").unwrap()),
            monitors: cfg.mon_list,
            result_papers: Vec::new(),
            with_wpaperd: cfg.with_wpaperd
        }
    }
    // split main image into two seperate, utilizes scaling
    pub fn run(mut self) -> Result<(), String> {
        // open original input image
        let mut img = image::open(&self.base_image_path).map_err(
            |err| err.to_string()
        )?;

        // calculate the hash
        self.base_hash = format!("# {:x}\n", compute(img.as_bytes()));

        // check existing config
        if self.with_wpaperd {
            let check_result = check_existing(
                format!("{}/.config/wpaperd/output.conf",var("HOME").unwrap()),
                &self.base_hash
            ).map_err(
                |err| err.to_string()
            )?;
            if check_result {
                // we're done
                return Ok(())
            }
        }

        // get base image details
        let (main_width, main_height) = img.dimensions();

        /*
            Calculate Overall Size
            Assuming the following configuration:

            +--------------+  +-----------------+
            |              |  |                 |
            |              |  |    primary      |
            |              |  |     monitor     |
            |  secondary   |  |                 |
            |    monitor   |  +-----------------+
            |              |
            |              |
            |              |
            +--------------+
            
            Assuming a monitor can never be negatively offset
            from root, we can say that max width will be the biggest monitor
            with the greatest x-offset, max height will be defined in the same
            way except using y-offset

            Should we ever get a negative offset, this will definitely panic ¯\_(ツ)_/¯
        */
        let mut overall_width = 0;
        let mut overall_height = 0;
        for monitor in &self.monitors {
            overall_width = cmp::max(monitor.width + monitor.x as u32, overall_width);
            overall_height = cmp::max(monitor.height + monitor.y as u32, overall_height);
        }

        // check if we need to upscale
        if overall_width > main_width || overall_height > main_height {
            // we need to scale
            println!(
                "{}: Scaling image to fit {}x{}",
                "WARNING".red().bold(),
                overall_width,
                overall_height
            );

            // upscale image to fit
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
                        "{}rwpspread_{}_{}x{}.png",
                        &self.save_path,
                        format!("{}", &monitor.name),
                        cropped_image.width(),
                        cropped_image.height(),
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
        if self.with_wpaperd {
            // create new wpaperd instance
            let wpaperd = WpaperdConfig::new(
                format!("{}/.config/wpaperd/output.conf", var("HOME").unwrap()),
                self.result_papers,
                self.base_hash
            );

            // build config
            wpaperd.build().map_err(
                |err| err.to_string()
            )?;
        }

        // return
        Ok(())
    }
}
