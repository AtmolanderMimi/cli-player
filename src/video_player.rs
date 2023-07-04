use std::error::Error;
use std::thread;
use std::time::Duration;

use crate::character_pallet::CharacterPallet;
use crate::video::Video;

pub async fn play_video(video: Video, pallet: &CharacterPallet, width: u32) -> Result<(), Box<dyn Error>> {
    let fps = video.fps();
    for frame in video.into_iter() {
        println!("{}", frame.as_string(&pallet, width));
        thread::sleep(Duration::from_secs(1) / fps)
    }
    Ok(())
}