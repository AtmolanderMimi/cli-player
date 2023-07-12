//! Defines how a video is played through the terminal

use std::error::Error;
use std::fmt::{Display, Debug};
use std::io;
use std::process::Command;

use rustube::Video as YtVideo;
use rustube::url::Url;

use crate::audio_manager::AudioManager;
use crate::config::Config;
use crate::image::ImageAsString;
use crate::frames::FramesManager;

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

/// Contains the data of the video and is responsible for downloading
pub struct Video {
    frames: FramesManager,
    audio_source_path: String,
    audio_player: AudioManager,
    _current_frame: usize,
}

impl Video {
    fn new(frames: FramesManager, audio_source_path: String, audio_player: AudioManager) -> Video {
        Video {
            frames,
            audio_source_path,
            audio_player,
            _current_frame: 0,
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
        const TEMP_AUDIO_PATH: &str = "./temp_audio.wav";
        
        let frames = FramesManager::build(path, config)?;

        // Seperates audio from video
        match std::fs::remove_file(TEMP_AUDIO_PATH) {
            Ok(()) => (),
            // It makes an error when there is no file to delete which is normal
            Err(_) => (),
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
            // We don't really care if the audio is invalid, it will get caught later and there will
            // just be no audio
            Err(_) => eprintln!("Error while using FFMPEG, is it installed?"),
            //Err(e) => return Err(VideoError::FfmpegError(e)),
        }

        let audio_player = AudioManager::build()?;

        let video = Video::new(frames, TEMP_AUDIO_PATH.to_string(), audio_player);
        Ok(video)
    }
}

impl Video {
    pub fn next_frame(&mut self) -> Option<Box<dyn ImageAsString>> {
        self.frames.next_frame()
    }

    pub fn next_frame_string(&mut self, config: &Config) -> Option<String> {
        let next_frame = self.next_frame()?;
        Some(next_frame.as_string(config))
    }

    /// Gives the fps of the video
    pub fn fps(&self) -> u32 {
        self.frames.fps()
    }

    pub fn start_audio(&self) -> Result<(), VideoError> {
        self.audio_player.play_from_path(&self.audio_source_path)
    }

    pub fn set_volume(&self, volume: f32) {
        self.audio_player.set_volume(volume)
    }
}

// TODO: Add tests