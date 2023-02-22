use image::{GenericImageView, DynamicImage, imageops::FilterType};
use std::cmp;
use colored::Colorize;
use rwpspread::Config;

// result paper struct
pub struct ResultPaper {
    pub name: String,
    pub image: DynamicImage,
}

// split main image into two seperate, utilizes scaling
pub fn split_image(config: Config) -> Result<Vec<ResultPaper>, &'static str> {
    // new vector for result imgs
    let mut result_papers: Vec<ResultPaper> = Vec::new();

    // open original input image
    let mut img = image::open(&config.image_file).map_err(|_| "")?;

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
        
        @TODO:
            -> max height is defined by primary or secondary
            -> max width is defined as sum of all
            ^ This will most likely fail if monitors are
            stacked or negatively aligned to root
    */
    let mut overall_width = 0;
    let mut overall_height = 0;
    for monitor in &config.mon_list {
        overall_height = cmp::max(monitor.height, overall_height);
        overall_width += monitor.width;
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
    for monitor in config.mon_list {
        let cropped_image = img.crop(
            monitor.x as u32,
            monitor.y as u32,
            monitor.width,
            monitor.height
        );
        result_papers.push(
            ResultPaper { 
                name: monitor.name,
                image: cropped_image
            }
        )
    }

    // return the vector
    Ok(result_papers)
}
