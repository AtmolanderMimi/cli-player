use std::process;

use cli_player::video::Video;
use cli_player::video_player;
use cli_player::config::Config;

#[tokio::main]
async fn main() {
    let config = match Config::build_from_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error while creating config: {e}");
            process::exit(1);
        }
    };

    println!("Processing...");
    let video = match Video::build_from_path(config.query(), &config) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error while tring to find on computer: {e}");

            println!("Downloading and processing...");
            match Video::build_from_url(config.query(), &config).await {
                Ok(v) => v,
                Err(e) => { eprint!("Error while tring to download: {e}"); process::exit(1); }
            }
        },
    };

    match video_player::play_video(video, &config).await {
        Ok(()) => (),
        Err(e) => eprintln!("Error while playing the video: {e}"),
    };
}
