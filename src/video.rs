//! Defines how a video is played through the terminal

use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Display, Debug};
use std::fs::File;
use std::io::{self, BufReader};
use std::process::Command;
use std::rc::Rc;

use opencv::core::UMat;
use opencv::videoio;
use opencv::videoio::{VideoCapture, VideoCaptureTrait};
use opencv::videostab::{VideoFileSourceTrait, VideoFileSource};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use rustube::Video as YtVideo;
use rustube::url::Url;
use rodio::{Decoder, OutputStream, Sink};

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

/// Contains the data of the video and is responsible for downloading
pub struct Video {
    frames: Vec<Rc<Box<dyn ImageAsString>>>,
    audio_source_path: String,
    audio_player: RefCell<Option<Sink>>,
    fps: u32,
    current_frame: usize,
}

impl Video {
    pub fn new(frames: Vec<Box<dyn ImageAsString>>, fps: u32, audio_source_path: String) -> Video {
        let frames = frames.into_iter().map(|f| Rc::new(f)).collect();
        Video {
            frames,
            fps,
            audio_source_path,
            audio_player: RefCell::new(None),
            current_frame: 0,
        }
    }

    /// Downloaded the video to ./downloaded-videos/ and collects all the frames
    pub async fn build_from_url(url: &str, config: &Config) -> Result<Video, VideoError> {
        println!("{}", url);
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

        // TODO: This piece of code is an absolute memory hog, so much so that the program cannot run with bigger videos while not using preprocessing
        // Consider streaming the frames in instead of having them all in memory
        let mut frames: Vec<Box<dyn ImageAsString>> = Vec::new();
        let mut buffer = UMat::new(opencv::core::UMatUsageFlags::USAGE_DEFAULT);
        let mut frame_chunk = Vec::new();
        while match capture.read(&mut buffer) {
            Ok(b) => b,
            Err(e) => return Err(VideoError::OpenCvError(e)),
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

        let video = Video::new(frames, fps, TEMP_AUDIO_PATH.to_string());
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
    fn audio_buffer_from_path(path: &str) -> Result<Decoder<BufReader<File>>, VideoError> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(VideoError::IoError(e)),
        };

        let buf_reader = BufReader::new(file);

        // "Error while playing the video: Unrecognized format" while trying to open .mp4 file
        let audio_source =  match Decoder::new(buf_reader) { 
            Ok(d) => d,
            Err(e) => return Err(VideoError::RodioError(RodioError::DecoderError(e)))
        };

        Ok(audio_source)
    }

    /// Gives the fps of the video
    pub fn fps(&self) -> u32 {
        self.fps
    }

    // TODO: Move audio system to it's own struct
    // Make sure to keep OutputStream in scope to play
    pub fn start_audio(&self) -> Result<OutputStream, VideoError> {
        // We NEED to keep _out_stream in scope for the audio to play
        let (out_stream, output_stream_handle) = match OutputStream::try_default() {
            Ok((_output, s)) => (_output, s),
            Err(e) => return Err(VideoError::RodioError(RodioError::StreamError(e)))
        };
        let audio_source = Video::audio_buffer_from_path(&self.audio_source_path)?;
        //let samples = audio_source.convert_samples();

        let new_sink = Sink::try_new(&output_stream_handle).unwrap();
        new_sink.append(audio_source);
        *self.audio_player.borrow_mut() = Some(new_sink);
        Ok(out_stream)
    }
}

// TODO: Add tests