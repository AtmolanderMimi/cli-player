use opencv::videoio::{VideoCapture, VideoCaptureTrait, self};
use opencv::core::UMat;
use opencv::videostab::{VideoFileSource, VideoFileSourceTrait};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{VideoError, Config};
use crate::image::{TextImage, Image, ImageAsString};

pub struct FramesManager {
    frames: Frames,
    target_fps: u32,
    error_per_frame: f64,            
    error: f64,
}

impl FramesManager {
    fn new(frames: Frames, fps: u32, mut target_fps: u32) -> FramesManager {
        if target_fps > fps {
            target_fps = fps;
        }

        let error_per_frame = (fps as f64 / target_fps as f64) - 1.0;

        FramesManager {
            frames,
            target_fps,
            error_per_frame,
            error: 0.0
        }
    }

    pub fn build(path: &str, config: &Config) -> Result<FramesManager, VideoError> {
        let mut source = match VideoFileSource::new(&path, false) {
            Ok(c) => c,
            Err(e) => return Err(VideoError::OpenCvError(e))
        };
        let fps = match source.fps() {
            Ok(f) => f as u32,
            Err(e) => return Err(VideoError::OpenCvError(e)),
        };

        let capture = match VideoCapture::from_file(&path, videoio::CAP_ANY) {
            Ok(c) => c,
            Err(e) => return Err(VideoError::OpenCvError(e))
        };

        let target_fps = match config.frame_limit() {
            0 => u32::MAX,
            x => x,
        };

        let frames = if config.preprocessing() {
            Frames::build_preprocessed(capture, config)?
        } else {
            Frames::build_streamed(capture)
        };

        let frames_manager = FramesManager::new(frames, fps, target_fps);

        Ok(frames_manager) 
    }
}

impl FramesManager {
    pub fn next_frame(&mut self) -> Option<Box<dyn ImageAsString>> {
        self.error += self.error_per_frame;

        while self.error >= 1.0 {
            self.frames.next_frame();
            self.error -= 1.0;
        }

        self.frames.next_frame()
    }

    pub fn fps(&self) -> u32 {
        self.target_fps
    }
}

enum Frames {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config_streamed() -> Config {
        Config::build("video.mp4".to_string(), "ascii".to_string(), 50, 15, 1.0, true, false)
            .unwrap()
    }

    fn default_config_preprocessed() -> Config {
        Config::build("video.mp4".to_string(), "ascii".to_string(), 50, 15, 1.0, true, true)
            .unwrap()
    }

    fn default_config_no_limiter() -> Config {
        Config::build("video.mp4".to_string(), "ascii".to_string(), 50, 0, 1.0, true, false)
            .unwrap()
    }

    fn default_config_big_limiter() -> Config {
        Config::build("video.mp4".to_string(), "ascii".to_string(), 50, 1, 1.0, true, false)
            .unwrap()
    }

    #[test]
    fn build_frames_from_path() {
        let config = &default_config_streamed();
        FramesManager::build("./test-assets/video.mp4", config).unwrap();
    }

    // If the videos are the same they will both try to get assess to the right to read
    #[test]
    fn get_images_frames_manager_streamed() {
        let config = &default_config_streamed();
        let mut frames = FramesManager::build("./test-assets/video-streamed.mp4", config).unwrap();
        frames.next_frame().unwrap();
    }

    #[test]
    fn get_images_frames_manager_preprocessed() {
        let config = &default_config_preprocessed();
        let mut frames = FramesManager::build("./test-assets/video-preprocessed.mp4", config).unwrap();
        frames.next_frame().unwrap();
    }

    #[test]
    fn no_frame_limit() {
        let config = &default_config_no_limiter();
        let mut frames = FramesManager::build("./test-assets/video-preprocessed.mp4", config).unwrap();
        frames.next_frame().unwrap();
    }

    #[test]
    fn big_frame_limit() {
        let config = &default_config_big_limiter();
        let mut frames = FramesManager::build("./test-assets/video-preprocessed.mp4", config).unwrap();
        frames.next_frame().unwrap();
    }
}