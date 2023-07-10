use std::error::Error;
use std::fmt::Display;
use std::thread;
use std::time::{Duration, SystemTime};

use crate::config::Config;
use crate::video::Video;

#[derive(Debug)]
pub enum VideoPlayerError {
    TooMuchLag,
}

impl Display for VideoPlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoPlayerError::TooMuchLag => write!(f, "Frames take too much time to render"),
        }
    }
}

impl Error for VideoPlayerError {}

pub async fn play_video(video: Video, config: &Config) -> Result<(), Box<dyn Error>> {
    let fps = video.fps();
    println!("FPS = {fps}");
    thread::sleep(Duration::from_secs(3));

    // Starts the audio
    // The audio will stop when the program stops or when it has no more audio
    video.start_audio()?;

    let mut lag_count: u32 = 0;
    for frame in video.into_iter() {
        let start = SystemTime::now();

        println!("{}", frame.as_string(&config));

        let delta_time = start.elapsed()?;
        let sleep_duration = match (Duration::from_secs(1) / fps).checked_sub(delta_time) {
            Some(d) => d,
            None => {
                lag_count += 4;
                Duration::from_secs(0)
            },
        };

        if lag_count >= 25 {
            return Err(Box::new(VideoPlayerError::TooMuchLag));
        } else {
            lag_count = lag_count.checked_sub(1).unwrap_or(0);
        }
        thread::sleep(sleep_duration);
    }
    Ok(())
}