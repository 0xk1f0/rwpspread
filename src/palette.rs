use image::{GenericImageView, Rgba};
use serde::Serialize;
use serde_json::to_writer_pretty;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

#[derive(Serialize)]
struct ColorData {
    index: usize,
    color: String,
}

pub struct Palette {
    pixels: Vec<Rgba<u8>>,
    colors: Vec<String>,
}

impl Palette {
    pub fn new(image_path: &PathBuf) -> Result<Self, String> {
        let pixels = Palette::extract_rgba_pixels(image_path).map_err(|err| err.to_string())?;
        Ok(Self {
            pixels,
            colors: Vec::with_capacity(16),
        })
    }

    fn extract_rgba_pixels(image_path: &PathBuf) -> Result<Vec<Rgba<u8>>, String> {
        // Load the image
        let img = image::open(image_path).map_err(|err| err.to_string())?;

        // Resize the image to a small size for less color diversion and faster processing
        let small_img = img.resize_exact(16, 16, image::imageops::FilterType::Nearest);

        // Collect the RGBA values of each pixel in a vector
        let mut pixels = Vec::with_capacity(16 * 16);
        for pixel in small_img.pixels() {
            pixels.push(pixel.2);
        }

        // return them
        Ok(pixels)
    }

    pub fn generate_mostused(mut self, save_path: String) -> Result<(), String> {
        // Create a HashMap to store the frequency of each color
        let mut color_counts: HashMap<Rgba<u8>, usize> = HashMap::new();

        // Loop over the vector to analyze pixels
        for entry in &self.pixels {
            // Count the frequency of each color
            *color_counts.entry(*entry).or_insert(0) += 1;
        }

        // Sort the colors by frequency in descending order
        let mut sorted_colors: Vec<(Rgba<u8>, usize)> = color_counts.into_iter().collect();
        sorted_colors.sort_by_key(|(_, count)| *count);
        sorted_colors.reverse();

        // Most used colors in hexadecimal notation
        let mut most_used_colors = sorted_colors
            .iter()
            .take(16)
            .map(|(color, _)| {
                let hex_channels = color
                    .0
                    .iter()
                    .take(3)
                    .map(|channel| format!("{:02x}", channel))
                    .collect::<Vec<String>>();
                let hex_string = format!("#{}", hex_channels.join(""));
                let mut values = color
                    .0
                    .iter()
                    .take(3)
                    .map(|&channel| channel as u32)
                    .collect::<Vec<u32>>();
                // this makes sure the highest val is the
                // third number in the vector
                values.sort();
                (hex_string, values[2])
            })
            .collect::<Vec<(String, u32)>>();

        // sort by max value reached by color
        most_used_colors.sort_by(|(_, avg_value1), (_, avg_value2)| {
            avg_value1.partial_cmp(avg_value2).unwrap()
        });
        self.colors = most_used_colors
            .iter()
            .map(|(color, _)| color.clone())
            .collect::<Vec<String>>();

        // out to json
        self.to_json(save_path).map_err(|err| err.to_string())?;

        // done
        Ok(())
    }

    fn to_json(&self, path: String) -> Result<(), String> {
        let color_data: Vec<ColorData> = self
            .colors
            .iter()
            .enumerate()
            .map(|(index, color)| ColorData {
                index: index + 1,
                color: color.to_string(),
            })
            .collect();

        let file = File::create(path).map_err(|err| err.to_string())?;
        to_writer_pretty(file, &color_data).map_err(|err| err.to_string())?;

        Ok(())
    }
}
