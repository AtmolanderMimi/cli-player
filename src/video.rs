//! Defines how a video is played through the terminal

use std::error::Error;
use std::fmt::{Display, Debug};
use std::rc::Rc;

use opencv::core::UMat;
use opencv::videoio;
use opencv::videoio::{VideoCapture, VideoCaptureTrait};
use opencv::videostab::{VideoFileSourceTrait, VideoFileSource};
use rayon::prelude::{IntoParallelRefIterator, IntoParallelIterator, ParallelIterator};
use rustube::Video as YtVideo;
use rustube::url::Url;

use crate::config::Config;
use crate::image::{TextImage, Image, ImageAsString};

#[derive(Debug)]
pub enum VideoParsingError {
    RustubeError(rustube::Error),
    OpenCvError(opencv::Error),
    UrlParseError(rustube::url::ParseError),
    NoStream,
}

impl Display for VideoParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoParsingError::RustubeError(e) => write!(f, "{}", e),
            VideoParsingError::OpenCvError(e) => write!(f, "{}", e),
            VideoParsingError::UrlParseError(e) => write!(f, "{}", e),
            VideoParsingError::NoStream => write!(
                f,
                "No stream of the video could be found",
            ),
        }
    }
}

impl Error for VideoParsingError {}

// NOTE: For now the frames will be rendered at while the video is displaying, but
// if live performance becomes an issue, the rendering of frames can be made beforehand

/// Contains the data of the video and is responsible for downloading
pub struct Video {
    frames: Vec<Rc<Box<dyn ImageAsString>>>,
    fps: u32,
    current_frame: usize,
}

impl Video {
    pub fn new(frames: Vec<Box<dyn ImageAsString>>, fps: u32) -> Video {
        let frames = frames.into_iter().map(|f| Rc::new(f)).collect();
        Video {
            frames,
            fps,
            current_frame: 0,
        }
    }

    /// Downloaded the video to ./downloaded-videos/ and collects all the frames
    pub async fn build_from_url(url: &str, config: &Config) -> Result<Video, VideoParsingError> {
        println!("{}", url);
        let url = match Url::parse(url) {
            Ok(u) => u,
            Err(e) => return Err(VideoParsingError::UrlParseError(e)),
        };
        let yt_video = match YtVideo::from_url(&url).await {
            Ok(v) => v,
            Err(e) => return Err(VideoParsingError::RustubeError(e)),
        };

        let stream = match yt_video.best_quality() {
            Some(s) => s,
            None => return Err(VideoParsingError::NoStream),
        };

        let video_path = match stream.download_to_dir("./downloaded-videos").await {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(e) => return Err(VideoParsingError::RustubeError(e)),
        };

        let video = Video::build_from_path(&video_path, config)?;
        Ok(video)
    }

    /// Collects all the frames from the video specified at the path
    pub fn build_from_path(path: &str, config: &Config) -> Result<Video, VideoParsingError> {
        const FRAME_CHUNK_SIZE: usize = 10;

        let mut source = match VideoFileSource::new(&path, false) {
            Ok(c) => c,
            Err(e) => return Err(VideoParsingError::OpenCvError(e))
        };
        let fps = match source.fps() {
            Ok(f) => f as u32,
            Err(e) => return Err(VideoParsingError::OpenCvError(e)),
        };

        let mut capture = match VideoCapture::from_file(&path, videoio::CAP_ANY) {
            Ok(c) => c,
            Err(e) => return Err(VideoParsingError::OpenCvError(e))
        };

        // TODO: This piece of code is an absolute memory hog, so much so that the program cannot run with bigger videos while not using preprocessing
        // Consider streaming the frames in instead of having them all in memory
        let mut frames: Vec<Box<dyn ImageAsString>> = Vec::new();
        let mut buffer = UMat::new(opencv::core::UMatUsageFlags::USAGE_DEFAULT);
        let mut frame_chunk = Vec::new();
        while match capture.read(&mut buffer) {
            Ok(b) => b,
            Err(e) => return Err(VideoParsingError::OpenCvError(e)),
        } {
            let frame = Image::new(buffer);

            if !config.preprocessing() {
                frames.push(Box::new(frame));
            } else {
                frame_chunk.push(frame);

                if frame_chunk.len() == FRAME_CHUNK_SIZE {
                    let text_images = frame_chunk.into_par_iter()
                        .map(|f| Box::new(TextImage::build_from_image(f, &config)))
                        .collect::<Vec<Box<TextImage>>>();

                    text_images.into_iter().for_each(|ti| frames.push(ti));
                    frame_chunk = Vec::new();
                }
            }
            buffer = UMat::new(opencv::core::UMatUsageFlags::USAGE_DEFAULT);
        }

        // Processes the frame chunk that was not complete
        let text_images = frame_chunk.into_par_iter()
        .map(|f| Box::new(TextImage::build_from_image(f, &config)))
        .collect::<Vec<Box<TextImage>>>();

        text_images.into_iter().for_each(|ti| frames.push(ti));

        let video = Video::new(frames, fps);
        Ok(video)
    }
}

impl Iterator for Video {
    type Item = Rc<Box<dyn ImageAsString>>;

    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.frames.get(self.current_frame)?;
        let frame = Rc::clone(frame);
        self.current_frame += 1;

        Some(frame)
    }
}

impl Video {
    /// Downloaded the video to ./downloaded-videos/ and collects all the frames
    pub fn fps(&self) -> u32 {
        self.fps
    }
}

// TODO: Add tests