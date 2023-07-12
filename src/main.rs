use std::process;

use cli_player::{Video, Config};
use cli_player::video_player;
use cli_player::wating_animation;

#[tokio::main]
async fn main() {
    let config = match Config::build_from_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error while creating config: {e}");
            process::exit(1);
        }
    };

    let animation = wating_animation::spawn_animation_thread("Processing");
    let video = match Video::build_from_path(config.query(), &config) {
        Ok(v) => { animation.end(); v },
        Err(e) => {
            animation.end();
            eprintln!("Error while tring to find on computer: {e}");

            let animation = wating_animation::spawn_animation_thread("Downloading and processing");
            match Video::build_from_url(config.query(), &config).await {
                Ok(v) => { animation.end(); v },
                Err(e) => { eprint!("Error while tring to download: {e}"); process::exit(1); }
            }
        },
    };

    match video_player::play_video(video, &config).await {
        Ok(()) => (),
        Err(e) => eprintln!("Error while playing the video: {e}"),
    };
}
