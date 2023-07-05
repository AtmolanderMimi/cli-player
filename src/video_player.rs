use std::error::Error;
use std::fmt::Display;
use std::thread;
use std::time::{Duration, SystemTime};

use crate::character_pallet::CharacterPallet;
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

pub async fn play_video(video: Video, pallet: &CharacterPallet, width: u32, color: bool) -> Result<(), Box<dyn Error>> {
    let fps = video.fps();

    for frame in video.into_iter() {
        let start = SystemTime::now();

        println!("{}", frame.as_string(&pallet, width, color));

        let delta_time = start.elapsed()?;
        let sleep_duration = match (Duration::from_secs(1) / fps).checked_sub(delta_time) {
            Some(d) => d,
            None => return Err(Box::new(VideoPlayerError::TooMuchLag)),
        };
        thread::sleep(sleep_duration);
    }
    Ok(())
}