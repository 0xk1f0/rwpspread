use image::{GenericImageView, Rgba};
use serde::Serialize;
use serde_json::{to_writer_pretty, Map, Value};
use std::collections::HashMap;
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
        let pixels = Palette::extract_rgba_pixels(image_path).map_err(|err| err)?;
        Ok(Self {
            path: image_path.to_string_lossy().to_string(),
            pixels,
            colors: Vec::with_capacity(16),
        })
    }

    fn extract_rgba_pixels(image_path: &PathBuf) -> Result<Vec<Rgba<u8>>, String> {
        // Load the image
        let img = image::open(image_path).map_err(|err| err.to_string())?;

        // determine resolution and downscale divisor
        let (width, height) = img.dimensions();
        // @TODO: Might make this changeable by user in the future
        let downscale_divisor: f64 = (width.max(height) as f64 / 750.0).max(1.0);

        // calculate new values and round
        let new_width = (width as f64 / downscale_divisor).round() as u32;
        let new_height = (height as f64 / downscale_divisor).round() as u32;

        // Resize the image for faster processing
        let small_img =
            img.resize_exact(new_width, new_height, image::imageops::FilterType::Nearest);

        // Collect the RGBA values of each pixel in a vector
        let mut pixels = Vec::with_capacity((new_width * new_height) as usize);
        for pixel in small_img.pixels() {
            pixels.push(pixel.2);
        }

        // return them
        Ok(pixels)
    }

    fn gamma_correct(&self, input: u8) -> f64 {
        // approximate gamma correction for sRGB range
        let gamma = 2.2;
        let linear = (input as f64 / 255.0).powf(gamma);
        linear
    }

    fn relative_luminance(&self, input: (u8, u8, u8)) -> f64 {
        // calculate approximate relative luminance
        (0.2126 * self.gamma_correct(input.0))
            + (0.7152 * self.gamma_correct(input.1))
            + (0.0722 * self.gamma_correct(input.2))
    }

    fn upshade_for_range(
        &self,
        last_color: (u8, u8, u8),
        min_lumin: f64,
        max_lumin: f64,
    ) -> (u8, u8, u8) {
        let mut luminance = self.relative_luminance(last_color);
        // step up the color until the target luminance range is met
        // this seems quite messy but has to be saturating add because
        // we have to iterate the other colors further until all have reached
        // 255, which is the brightest we can go for each channel
        let mut steps_taken: u8 = 0;
        while luminance >= max_lumin
            || luminance <= min_lumin
            && !(last_color.0 == u8::MAX && last_color.1 == u8::MAX && last_color.2 == u8::MAX)
        {
            steps_taken += 1;
            luminance = self.relative_luminance((
                last_color.0.saturating_add(steps_taken),
                last_color.1.saturating_add(steps_taken),
                last_color.2.saturating_add(steps_taken),
            ));
        }

        // return the modified color as a result
        (
            last_color.0.saturating_add(steps_taken),
            last_color.1.saturating_add(steps_taken),
            last_color.2.saturating_add(steps_taken),
        )
    }

    pub fn generate_mostused(mut self, save_path: &String) -> Result<(), String> {
        // define 16 luminance sections
        let luminance_boundaries = [
            0.0, 0.0625, 0.125, 0.1875, 0.25, 0.3125, 0.375, 0.4375, 0.5, 0.5625, 0.625, 0.6875,
            0.75, 0.8125, 0.875, 0.9375, 1.0,
        ];

        // generate a color to count hashmap
        let mut color_map: HashMap<Rgba<u8>, usize> = HashMap::new();
        for x in &self.pixels {
            *color_map.entry(*x).or_default() += 1;
        }

        // sort by color frequency
        let mut count_vec: Vec<(Rgba<u8>, usize)> = color_map.into_iter().collect();
        count_vec.sort_by(|a, b| b.1.cmp(&a.1));

        // append them by checking their luminance
        // and keep track of last color
        // if no suitable color is found, reshade the last one
        let mut last_color = (0, 0, 0);
        (0..16)
            .into_iter()
            .map(|num| {
                // find color with relative luminance calculation
                let chosen_color: (u8, u8, u8) = count_vec
                    .iter()
                    .find(|&&color| {
                        let this_lumin: f64 =
                            self.relative_luminance((color.0[0], color.0[1], color.0[2]));
                        this_lumin > luminance_boundaries[num]
                            && this_lumin < luminance_boundaries[num + 1]
                    })
                    .map_or_else(
                        || {
                            self.upshade_for_range(
                                last_color,
                                luminance_boundaries[num],
                                luminance_boundaries[num + 1],
                            )
                        },
                        |color| (color.0[0], color.0[1], color.0[2]),
                    );
                // push it to the result vector
                self.colors.push(ColorData {
                    index: num,
                    color: format!(
                        "#{:02X}{:02X}{:02X}",
                        chosen_color.0, chosen_color.1, chosen_color.2
                    ),
                });
                // remember color
                last_color = chosen_color;
            })
            .count();

        // save to json
        self.to_json(save_path.to_string()).map_err(|err| err)?;

        // done
        Ok(())
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
