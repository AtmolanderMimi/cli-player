use std::error::Error;
use std::thread;
use std::time::{Duration, SystemTime};

use crate::character_pallet::CharacterPallet;
use crate::video::Video;

pub async fn play_video(video: Video, pallet: &CharacterPallet, width: u32, color: bool) -> Result<(), Box<dyn Error>> {
    let fps = video.fps();

    for frame in video.into_iter() {
        let start = SystemTime::now();

        println!("{}", frame.as_string(&pallet, width, color));

        let delta_time = start.elapsed()?;
        thread::sleep((Duration::from_secs(1) / fps) - delta_time);
    }
    Ok(())
}