use opencv::videoio::{VideoCapture, VideoCaptureTrait};
use opencv::core::UMat;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{VideoError, Config};
use crate::image::{TextImage, Image, ImageAsString};

pub enum Frames {
    Streamed(VideoCapture),
    Preprocessed(Vec<TextImage>)
}

impl Frames {
    pub fn build_preprocessed(mut capture: VideoCapture, config: &Config) -> Result<Frames, VideoError> {
        const FRAME_CHUNK_SIZE: usize = 10;

        let mut frames: Vec<TextImage> = Vec::new();
        let mut buffer = UMat::new(opencv::core::UMatUsageFlags::USAGE_DEFAULT);
        let mut frame_chunk = Vec::new();

        while match capture.read(&mut buffer) {
            Ok(b) => b,
            Err(e) => return Err(VideoError::OpenCvError(e)),
        } {
            let frame = Image::new(buffer);
        
            frame_chunk.push(frame);
        
            if frame_chunk.len() == FRAME_CHUNK_SIZE {
                let text_images = frame_chunk.into_par_iter()
                    .map(|f| TextImage::build_from_image(f, &config))
                    .collect::<Vec<TextImage>>();
            
                text_images.into_iter().for_each(|ti| frames.push(ti));
                frame_chunk = Vec::new();
            }
            
            buffer = UMat::new(opencv::core::UMatUsageFlags::USAGE_DEFAULT);
        }

        // Processes the frame chunk that was not complete
        let text_images = frame_chunk.into_par_iter()
        .map(|f| TextImage::build_from_image(f, &config))
        .collect::<Vec<TextImage>>();

        text_images.into_iter().for_each(|ti| frames.push(ti));

        Ok(Frames::Preprocessed(frames))
    }

    pub fn build_streamed(capture: VideoCapture) -> Frames {
        Frames::Streamed(capture)
    }
}

impl Frames {
    pub fn next_frame(&mut self) -> Option<Box<dyn ImageAsString>> {

        match self {
            Frames::Streamed(cap) => {
                let mut buffer = UMat::new(opencv::core::UMatUsageFlags::USAGE_DEFAULT);
                match cap.read(&mut buffer) {
                    Ok(_) => (),
                    Err(_) => return None,
                };

                Some(Box::new(Image::new(buffer)))
            },

            Frames::Preprocessed(frames) => {
                let current_frame = frames.remove(0);
                Some(Box::new(current_frame))
            }
        }
    }
}