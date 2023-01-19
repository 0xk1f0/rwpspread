use image::{GenericImageView, DynamicImage};
use num::rational::Ratio;

// simplify monitor ratio
fn simplify_ratio(pixel1: u32, pixel2: u32) -> (u32, u32) {
    let ratio = Ratio::new(pixel1, pixel2);
    (*ratio.numer(), *ratio.denom())
}

// determine image ratio for offset
fn determine_ratio(pixel1: u32, pixel2: u32) -> f64 {
    let ratio = pixel1 as f64 / pixel2 as f64;
    ratio
}

// parse cli input resolution
pub fn parse_resolution(resolution: String) -> Result<(u32, u32), &'static str> {
    let parts: Vec<&str> = resolution.split("x").collect();
    let width = parts[0].parse::<u32>().map_err(|_| "")?;
    let height = parts[1].parse::<u32>().map_err(|_| "")?;

    Ok((width, height))
}

// split main image into two seperate
pub fn split_image(input: &str, primary: &(u32, u32), secondary: &(u32, u32), offset: &u32) -> Result<Vec<DynamicImage>, &'static str> {
    // new vector for result imgs
    let mut result_papers: Vec<DynamicImage> = Vec::new();

    // open original input image
    let mut img = image::open(input).map_err(|_| "")?;

    // get base image dimensions
    let (main_width, main_height) = img.dimensions();

    // get monitor aspect ratios
    let asp_primary = simplify_ratio(primary.0, primary.1);
    let asp_secondary = simplify_ratio(secondary.0, secondary.1);

    // ratio relations
    let ratio_part_width = main_width / ( asp_primary.0 + asp_secondary.1 );
    let ratio_part_height = main_height / asp_secondary.0;

    // correctly size offset
    let offset_ratiod: u32 = (*offset as f64 * determine_ratio(main_height, secondary.1)).ceil() as u32;

    // Crop image for horizontal screen
    let primary_img = img.crop(
        ratio_part_width * asp_secondary.1,
        offset_ratiod,
        ratio_part_width * asp_primary.0,
        ratio_part_height * asp_primary.1
    );

    // Crop image for vertical screen
    let secondary_img = img.crop(
        0,
        0,
        ratio_part_width * asp_secondary.1,
        ratio_part_height * asp_secondary.0
    );

    // Push images to result vector
    result_papers.push(primary_img);
    result_papers.push(secondary_img);

    // return the vector
    Ok(result_papers)
}
