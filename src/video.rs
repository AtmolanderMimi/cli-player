//! Defines how a video is played through the terminal

use std::error::Error;
use std::fmt::{Display, Debug};

use opencv::core::Mat;
use opencv::videoio;
use opencv::videoio::{VideoCapture, VideoCaptureTrait};
use opencv::videostab::{VideoFileSourceTrait, VideoFileSource};
use rustube::Video as YtVideo;
use rustube::url::Url;

use crate::image::Image;

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

/// Contains the frames of the video and is responsible for downloading
pub struct Video {
    frames: Vec<Image>,
    fps: u32,
}

impl Video {
    pub fn new(frames: Vec<Image>, fps: u32) -> Video {
        Video {
            frames,
            fps
        }
    }
}

impl Video {
    /// Downloaded the video to ./downloaded-videos/ and collects all the frames
    pub async fn build_from_url(url: &str) -> Result<Video, VideoParsingError> {
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

        let mut source = match VideoFileSource::new(&video_path, false) {
            Ok(c) => c,
            Err(e) => return Err(VideoParsingError::OpenCvError(e))
        };
        let fps = match source.fps() {
            Ok(f) => f as u32,
            Err(e) => return Err(VideoParsingError::OpenCvError(e)),
        };

        let mut capture = match VideoCapture::from_file(&video_path, videoio::CAP_ANY) {
            Ok(c) => c,
            Err(e) => return Err(VideoParsingError::OpenCvError(e))
        };

        // "Why?" you may ask. Because OpenVC's image types are too much for me to handle
        let mut frames = Vec::new();
        let mut buffer = Mat::default();
        while match capture.read(&mut buffer) {
            Ok(b) => b,
            Err(e) => return Err(VideoParsingError::OpenCvError(e)),
        } {
            let frame = Image::new(buffer);
            frames.push(frame);
            buffer = Mat::default();
        }

        let video = Video::new(frames, fps);
        Ok(video)
    }
}

// TODO: Add tests