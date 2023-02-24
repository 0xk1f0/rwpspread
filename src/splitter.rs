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
        
        Assuming a monitor can never be negatively offset
        from root, we can say that max width will be the biggest monitor
        with the greatest x-offset, max height will be defined in the same
        way except using y-offset

        Should we ever get a negative offset, this will definitely panic ¯\_(ツ)_/¯
    */
    let mut overall_width = 0;
    let mut overall_height = 0;
    for monitor in &config.mon_list {
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
