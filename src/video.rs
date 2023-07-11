//! Defines how a video is played through the terminal

use std::error::Error;
use std::fmt::{Display, Debug};
use std::io;
use std::process::Command;

use opencv::core::UMat;
use opencv::videoio;
use opencv::videoio::{VideoCapture, VideoCaptureTrait};
use opencv::videostab::{VideoFileSourceTrait, VideoFileSource};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use rustube::Video as YtVideo;
use rustube::url::Url;

use crate::audio_manager::AudioManager;
use crate::config::Config;
use crate::image::{TextImage, Image, ImageAsString};

#[derive(Debug)]
pub enum VideoError {
    RustubeError(rustube::Error),
    OpenCvError(opencv::Error),
    UrlParseError(rustube::url::ParseError),
    IoError(io::Error),
    RodioError(RodioError),
    FfmpegError(io::Error),
    NoStream,
}


impl Display for VideoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoError::RustubeError(e) => write!(f, "{}", e),
            VideoError::OpenCvError(e) => write!(f, "{}", e),
            VideoError::UrlParseError(e) => write!(f, "{}", e),
            VideoError::IoError(e) => write!(f, "{}", e),
            VideoError::RodioError(e) => write!(f, "{}", e),
            VideoError::FfmpegError(e) => write!(f, "{}", e),
            VideoError::NoStream => write!(
                f,
                "No stream of the video could be found",
            ),
        }
    }
}

impl Error for VideoError {}

#[derive(Debug)]
pub enum RodioError {
    DecoderError(rodio::decoder::DecoderError),
    PlayError(rodio::PlayError),
    StreamError(rodio::StreamError),
}

impl Display for RodioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RodioError::DecoderError(e) => write!(f, "{}", e),
            RodioError::PlayError(e) => write!(f, "{}", e),
            RodioError::StreamError(e) => write!(f, "{}", e),
        }
    }
}

impl Error for RodioError {}

// NOTE: For now the frames will be rendered at while the video is displaying, but
// if live performance becomes an issue, the rendering of frames can be made beforehand

enum Frames {
    Streamed(VideoCapture),
    Preprocessed(Vec<TextImage>)
}

/// Contains the data of the video and is responsible for downloading
pub struct Video {
    frames: Frames,
    audio_source_path: String,
    audio_player: AudioManager,
    fps: u32,
    current_frame: usize,
}

impl Video {
    fn new(frames: Frames, fps: u32, audio_source_path: String, audio_player: AudioManager) -> Video {
        Video {
            frames,
            fps,
            audio_source_path,
            audio_player: audio_player,
            current_frame: 0,
        }
    }

    /// Downloaded the video to ./downloaded-videos/ and collects all the frames
    pub async fn build_from_url(url: &str, config: &Config) -> Result<Video, VideoError> {
        let url = match Url::parse(url) {
            Ok(u) => u,
            Err(e) => return Err(VideoError::UrlParseError(e)),
        };
        let yt_video = match YtVideo::from_url(&url).await {
            Ok(v) => v,
            Err(e) => return Err(VideoError::RustubeError(e)),
        };

        let stream = match yt_video.best_quality() {
            Some(s) => s,
            None => return Err(VideoError::NoStream),
        };

        let video_path = match stream.download_to_dir("./downloaded-videos").await {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(e) => return Err(VideoError::RustubeError(e)),
        };

        let video = Video::build_from_path(&video_path, config)?;
        Ok(video)
    }

    /// Collects all the frames from the video specified at the path
    pub fn build_from_path(path: &str, config: &Config) -> Result<Video, VideoError> {
        const FRAME_CHUNK_SIZE: usize = 10;
        const TEMP_AUDIO_PATH: &str = "./temp_audio.wav";

        let mut source = match VideoFileSource::new(&path, false) {
            Ok(c) => c,
            Err(e) => return Err(VideoError::OpenCvError(e))
        };
        let fps = match source.fps() {
            Ok(f) => f as u32,
            Err(e) => return Err(VideoError::OpenCvError(e)),
        };

        let mut capture = match VideoCapture::from_file(&path, videoio::CAP_ANY) {
            Ok(c) => c,
            Err(e) => return Err(VideoError::OpenCvError(e))
        };
        
        let frames = if config.preprocessing() {
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

            Frames::Preprocessed(frames)
        } else {
            Frames::Streamed(capture)
        };

        

        // Seperates audio from video
        match std::fs::remove_file(TEMP_AUDIO_PATH) {
            Ok(()) => (),
            Err(e) => return Err(VideoError::IoError(e))
        }

        let command_result = Command::new("ffmpeg")
            .args([
                "-i",
                path,
                "-vn",
                TEMP_AUDIO_PATH,
                ]).output();
                
        match command_result {
            Ok(_) => (),
            Err(e) => return Err(VideoError::FfmpegError(e)),
        }

        let audio_player = AudioManager::build()?;

        let video = Video::new(frames, fps, TEMP_AUDIO_PATH.to_string(), audio_player);
        Ok(video)
    }
}

impl Video {
    pub fn next_frame(&mut self) -> Option<Box<dyn ImageAsString>> {
        self.current_frame += 1;

        match &mut self.frames {
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

    pub fn next_frame_string(&mut self, config: &Config) -> Option<String> {
        let next_frame = self.next_frame()?;
        Some(next_frame.as_string(config))
    }

    /// Gives the fps of the video
    pub fn fps(&self) -> u32 {
        self.fps
    }

    pub fn start_audio(&self) -> Result<(), VideoError> {
        self.audio_player.play_from_path(&self.audio_source_path)
    }

    pub fn set_volume(&self, volume: f32) {
        self.audio_player.set_volume(volume)
    }
}

// TODO: Add tests