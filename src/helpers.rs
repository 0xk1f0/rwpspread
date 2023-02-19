use image::{GenericImageView, DynamicImage, imageops::FilterType};
use num::rational::Ratio;
use std::cmp;
use colored::Colorize;

// simplify monitor ratio
fn simplify_ratio(pixel1: u32, pixel2: u32) -> (u32, u32) {
    let ratio = Ratio::new(pixel1, pixel2);
    (*ratio.numer(), *ratio.denom())
}

// parse cli input resolution
pub fn parse_resolution(resolution: String) -> Result<(u32, u32), &'static str> {
    if resolution.contains("x") {
        let parts: Vec<&str> = resolution.split("x").collect();
        let width = parts[0].parse::<u32>().map_err(|_| "")?;
        let height = parts[1].parse::<u32>().map_err(|_| "")?;

        Ok((width, height))
    } else {
        Err("Parsing Error")
    }
}

// split main image into two seperate, utilizes scaling
pub fn split_image(input: &str, primary: &(u32, u32), secondary: &(u32, u32), offset: u32) -> Result<Vec<DynamicImage>, &'static str> {
    // new vector for result imgs
    let mut result_papers: Vec<DynamicImage> = Vec::new();

    // open original input image
    let mut img = image::open(input).map_err(|_| "")?;

    // get base image details
    let (main_width, main_height) = img.dimensions();
    let primary_aspect = simplify_ratio(primary.0, primary.1);
    let secondary_aspect = simplify_ratio(secondary.0, secondary.1);

    // info proc output
    println!(
        "Primary: ({}:{}) Secondary ({}:{})",
        primary_aspect.0,
        primary_aspect.1,
        secondary_aspect.0,
        secondary_aspect.1,
    );

    // calculate overall size
    /*
        Assuming a configuration where monitors are side on side:
        
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

        So either primary or secondary defines max height
        Width will always be the sum
    */
    let overall_width = primary.0 + secondary.0;
    let overall_height = cmp::max(primary.0, secondary.0);

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

    // Crop image for horizontal screen
    let primary_img = img.crop(
        secondary.0,
        offset,
        primary.0,
        primary.1
    );

    // Crop image for vertical screen
    let secondary_img = img.crop(
        0,
        0,
        secondary.0,
        secondary.1
    ); 

    // Push images to result vector
    result_papers.push(primary_img);
    result_papers.push(secondary_img);

    // return the vector
    Ok(result_papers)
}
