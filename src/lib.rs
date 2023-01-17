use std::error::Error;
use image::{GenericImageView, DynamicImage};

pub struct Config {
    pub image_file: String,
    pub offset: u32
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &str> {
        // handle arg count
        if args.len() < 3 {
            return Err("not enough arguments");
        } else if args.len() > 4 {
            return Err("too many arguments");
        } else if args[2].parse::<u32>().is_err() {
            return Err("offset is non-int");
        }

        // clone args to vars
        let image_file: String = args[1].clone();
        let offset: u32 = args[2].clone().trim().parse().unwrap();

        Ok(Config {
            image_file,
            offset
        })
    }
}

// determine image ratio for offset
fn determine_ratio(pixel1: u32, pixel2: u32) -> f64 {
    let ratio = pixel1 as f64 / pixel2 as f64;
    ratio
}

// split main image into two seperate
// @TODO: needs actual monitors sizes as input
fn split_image(input: &str, offset: &u32) -> Vec<DynamicImage> {
    let mut result_papers: Vec<DynamicImage> = Vec::new();
    let mut img = image::open(input).unwrap();

    // get base image dimensions
    let (width, height) = img.dimensions();

    // correctly size offset
    let offset_ratiod: u32 = (*offset as f64 * determine_ratio(height, 1920)).ceil() as u32;
    println!("{}", offset_ratiod);

    // Crop image for horizontal screen
    let horizontal_img = img.crop(
        (width/25) * 9,
        ((height/16) * 7) - offset_ratiod,
        (width/25) * 16,
        (height/16) * 9
    );

    // Crop image for vertical screen
    let vertical_img = img.crop(
        0,
        0,
        (width/25) * 9,
        height
    );

    // Push images to result vector
    result_papers.push(horizontal_img);
    result_papers.push(vertical_img);

    // return the vector
    result_papers
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {

    let images = split_image(&config.image_file, &config.offset);

    for image in images {
        let image_name = format!(
            "out_img_{}x{}_{}.png",
            image.width(),
            image.height(),
            &config.offset
        );
        image.save(image_name).unwrap();
    }

    Ok(())
}
