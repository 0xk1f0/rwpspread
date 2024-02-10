use image::{GenericImageView, Rgba};
use serde::Serialize;
use serde_json::{to_writer_pretty, Map, Value};
use std::fs::File;
use std::path::PathBuf;

#[derive(Serialize)]
struct PaletteFormat {
    wallpaper: String,
    foreground: String,
    background: String,
    colors: Map<String, Value>,
}

#[derive(Serialize)]
struct ColorData {
    index: usize,
    color: String,
}

pub struct Palette {
    path: String,
    pixels: Vec<Rgba<u8>>,
    colors: Vec<ColorData>,
}

impl Palette {
    pub fn new(image_path: &PathBuf) -> Result<Self, String> {
        let pixels = Palette::extract_rgba_pixels(image_path).map_err(|err| err.to_string())?;
        Ok(Self {
            path: image_path.to_string_lossy().to_string(),
            pixels,
            colors: Vec::with_capacity(16),
        })
    }

    pub fn generate_mostused(mut self, save_path: &String) -> Result<(), String> {
        // define 16 colors sections
        let sections = vec![
            (0..16, 0..16),
            (16..32, 16..32),
            (32..48, 32..48),
            (48..64, 48..64),
            (64..80, 64..80),
            (80..96, 80..96),
            (96..112, 96..112),
            (112..128, 112..128),
            (128..144, 128..144),
            (144..160, 144..160),
            (160..176, 160..176),
            (176..192, 176..192),
            (192..208, 192..208),
            (208..224, 208..224),
            (224..240, 224..240),
            (240..256, 240..256),
        ];

        // vectors for count and median sums
        let mut median_sums = Vec::new();
        let mut counts = Vec::new();

        // fill them initially
        for _ in 0..16 {
            median_sums.push((0, 0, 0));
            counts.push(0);
        }

        // accumulate sums for every section
        for rgb in &self.pixels {
            let r = rgb[0] as u32;
            let g = rgb[1] as u32;
            let b = rgb[2] as u32;

            for (i, (r_range, g_range)) in sections.iter().enumerate() {
                if r_range.contains(&r) && g_range.contains(&g) {
                    let (r_sum, g_sum, b_sum) = median_sums[i];
                    median_sums[i] = (r_sum + r, g_sum + g, b_sum + b);
                    counts[i] += 1;
                }
            }
        }

        // calculate the median for every section
        for i in 0..16 {
            let (r_sum, g_sum, b_sum) = median_sums[i];
            let count = counts[i];
            let median_r = if count > 0 { r_sum / count } else { 0 };
            let median_g = if count > 0 { g_sum / count } else { 0 };
            let median_b = if count > 0 { b_sum / count } else { 0 };
            let median_hex = format!("#{:02X}{:02X}{:02X}", median_r, median_g, median_b);

            self.colors.push(ColorData {
                index: i,
                color: median_hex,
            });
        }

        // save to json
        self.to_json(save_path.to_string())
            .map_err(|err| err.to_string())?;

        // done
        Ok(())
    }

    fn extract_rgba_pixels(image_path: &PathBuf) -> Result<Vec<Rgba<u8>>, String> {
        // Load the image
        let img = image::open(image_path).map_err(|err| err.to_string())?;

        // Resize the image to a small size for less color diversion and faster processing
        let small_img = img.resize_exact(256, 256, image::imageops::FilterType::Nearest);

        // Collect the RGBA values of each pixel in a vector
        let mut pixels = Vec::with_capacity(256 * 256);
        for pixel in small_img.pixels() {
            pixels.push(pixel.2);
        }

        // return them
        Ok(pixels)
    }

    fn to_json(self, path: String) -> Result<(), String> {
        let mut color_map = Map::new();

        for color in &self.colors {
            color_map.insert(
                format!("color{}", color.index),
                Value::String((*color.color).to_string()),
            );
        }

        let json_output = PaletteFormat {
            wallpaper: self.path,
            foreground: (*self.colors.last().expect("Palette Error").color).to_string(),
            background: (*self.colors.first().expect("Palette Error").color).to_string(),
            colors: color_map,
        };

        let file =
            File::create(format!("{}/rwps_colors.json", path)).map_err(|err| err.to_string())?;
        to_writer_pretty(file, &json_output).map_err(|err| err.to_string())?;

        Ok(())
    }
}
