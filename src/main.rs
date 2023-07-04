use std::env;

use cli_player::character_pallet;
use cli_player::video::Video;
use cli_player::video_player;

#[tokio::main]
async fn main() {
    let character_pallets =
        match character_pallet::parse_pallets_from_file("character-pallets.txt") {
            Ok(p) => p,
            Err(e) => panic!("Error while parseing: {e}"),
    };

    let url = env::args().nth(1).unwrap();
    println!("Downloading...");
    let video = Video::build_from_url(&url).await.unwrap();
    println!("Done!");
    video_player::play_video(video, &character_pallets[&"ascii".to_string()], 100).await.unwrap();
}
