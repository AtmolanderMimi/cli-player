use std::sync::Mutex;

use opencv::core::{UMat, Size, Point, VecN};
use opencv::imgproc;
use opencv::prelude::{UMatTraitConst, MatTraitConst};
use colored::Colorize;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::config::Config;

pub trait ImageAsString {
    fn as_string(&self, config: &Config) -> String;
}

/// Holds the contents of the Image
pub struct Image {
    content: UMat,
    as_string: Mutex<Option<String>>,
}

impl Image {
    pub fn new(content: UMat) -> Image {
        Image {
            content,
            as_string: Mutex::from(None),
        }
    }
}

impl Image {
    pub fn content(&self) -> &UMat {
        &self.content
    }

    fn scale(&self, width: u32) -> UMat {
        const INTERPOLATION: i32 = imgproc::INTER_LINEAR;
        const HEIGHT_TO_WIDHT: f64 = 2.0;

        let mut old_size = Size::default();
        let mut useless = Point::default();
        self.content.locate_roi(&mut old_size, &mut useless)
            .expect("Image should have size");

        let downscale_factor = width as f64 / old_size.width as f64;
        let height = ((old_size.height as f64 * downscale_factor) / HEIGHT_TO_WIDHT) as i32;
        let size = Size::from((width as i32, height));

        let mut scaled_image = UMat::new(opencv::core::UMatUsageFlags::USAGE_DEFAULT);
        // NOTE: See to change the interpolation for performance
        imgproc::resize(&self.content, &mut scaled_image, size, 0.0, 0.0, INTERPOLATION)
            .expect("Scaling should not fail given positive size");

        scaled_image
    }
}

impl ImageAsString for Image {
    // NOTE: Most of the lag of the program seems to come from this function
    fn as_string(&self, config: &Config) -> String {
        if let Some(s) = &*self.as_string.lock().unwrap() {
            return s.clone();
        }

        let scaled_image = self.scale(config.width());

        
        // Gets the rows
        let mut rows = Vec::new();
        for i in 0..scaled_image.rows() {
            let row = scaled_image.row(i)
            .expect("Row should not be out of range");
            let row = row.get_mat(opencv::core::AccessFlag::ACCESS_FAST).unwrap();
            rows.push(row);
        }

        // Render the rows in parralel
        let text_rows = rows.into_par_iter().map(|row| {
            let mut text_row = String::new();
            for x in 0..config.width() {
                let pixel: &VecN<u8, 3> = row.at(x as i32)
                    .expect("Pixel should not be out of range");

                let red = pixel[2];
                let green = pixel[1];
                let blue = pixel[0];

                let luminosity = ((red as u32 + green as u32 + blue as u32) / 3) as u8;
                let character = config.pallet().character_for_luminosity(luminosity).unwrap_or('�');

                if !config.color() {
                    text_row.push(character);
                } else {
                    let string = character.to_string();
                    let colored_string = string.truecolor(red, green, blue);
                    // Without this .to_string() the output is not colored
                    text_row.push_str(&colored_string.to_string());
                }
            }

            text_row.push('\n');

            text_row
        }).collect::<Vec<String>>();

        let mut out = String::new();
        text_rows.into_iter().for_each(|tr| out.push_str(&tr));

        *self.as_string.lock().unwrap() = Some(out);
        self.as_string(&config)
    }
}

/// Only stores the text representing the image
pub struct TextImage {
    text: String,
}

impl TextImage {
    fn new(text: String) -> TextImage {
        TextImage {
            text,
        }
    }

    pub fn build_from_image(image: Image, config: &Config) -> TextImage {
        let text = image.as_string(&config);
        TextImage::new(text)
    }
}

impl ImageAsString for TextImage {
    fn as_string(&self, _config: &Config) -> String {
        self.text.clone()
    }
}