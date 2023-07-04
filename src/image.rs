use opencv::core::{Mat, Size, Point, VecN};
use opencv::imgproc;
use opencv::prelude::MatTraitConst;

use crate::character_pallet::CharacterPallet;

/// Holds the contents of the Image
pub struct Image {
    content: Mat
}

impl Image {
    pub fn new(content: Mat) -> Image {
        Image {
            content
        }
    }
}

impl Image {
    pub fn content(&self) -> &Mat {
        &self.content
    }

    pub fn as_string(&self, pallet: &CharacterPallet, width: u32) -> String {
        let scaled_image = self.scale(width);

        let mut out = String::new();
        for y in 0..scaled_image.rows() {
            let row = scaled_image.row(y)
                .expect("Row should not be out of range");

            for x in 0..width {
                let pixel: &VecN<u8, 3> = row.at(x as i32)
                    .expect("Pixel should not be out of range");

                let luminosity = ((pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3) as u8;
                let character = pallet.character_for_luminosity(luminosity).unwrap_or('�');

                out.push(character);
            }

            out.push('\n');
        }

        out
    }

    fn scale(&self, width: u32) -> Mat {
        const INTERPOLATION: i32 = imgproc::INTER_LANCZOS4;
        const HEIGHT_TO_WIDHT: f64 = 2.3;

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