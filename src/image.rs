use std::sync::Mutex;

use opencv::core::{Mat, Size, Point, VecN};
use opencv::imgproc;
use opencv::prelude::MatTraitConst;
use colored::Colorize;

use crate::character_pallet::CharacterPallet;

/// Holds the contents of the Image
pub struct Image {
    content: Mat,
    as_string: Mutex<Option<String>>,
}

impl Image {
    pub fn new(content: Mat) -> Image {
        Image {
            content,
            as_string: Mutex::from(None),
        }
    }
}

impl Image {
    pub fn content(&self) -> &Mat {
        &self.content
    }

    pub fn as_string(&self, pallet: &CharacterPallet, width: u32, color: bool) -> String {
        if let Some(s) = &*self.as_string.lock().unwrap() {
            return s.clone();
        }

        let scaled_image = self.scale(width);

        let mut out = String::new();
        for y in 0..scaled_image.rows() {
            let row = scaled_image.row(y)
                .expect("Row should not be out of range");

            for x in 0..width {
                let pixel: &VecN<u8, 3> = row.at(x as i32)
                    .expect("Pixel should not be out of range");

                let red = pixel[2];
                let green = pixel[1];
                let blue = pixel[0];

                let luminosity = ((red as u32 + green as u32 + blue as u32) / 3) as u8;
                let character = pallet.character_for_luminosity(luminosity).unwrap_or('�');

                if !color {
                    out.push(character);
                } else {
                    let string = character.to_string();
                    let colored_string = string.truecolor(red, green, blue);
                    out = format!("{}{}", out, colored_string);
                }
            }

            out.push('\n');
        }

        *self.as_string.lock().unwrap() = Some(out);
        self.as_string(pallet, width, color)
    }

    fn scale(&self, width: u32) -> Mat {
        const INTERPOLATION: i32 = imgproc::INTER_LANCZOS4;
        const HEIGHT_TO_WIDHT: f64 = 2.0;

        let mut old_size = Size::default();
        let mut useless = Point::default();
        self.content.locate_roi(&mut old_size, &mut useless)
            .expect("Image should have size");

        let downscale_factor = width as f64 / old_size.width as f64;
        let height = ((old_size.height as f64 * downscale_factor) / HEIGHT_TO_WIDHT) as i32;
        let size = Size::from((width as i32, height));

        let mut scaled_image = Mat::default();
        // INFO: See to change the interpolation for performance
        imgproc::resize(&self.content, &mut scaled_image, size, 0.0, 0.0, INTERPOLATION)
            .expect("Scaling should not fail given positive size");

        scaled_image
    }
}