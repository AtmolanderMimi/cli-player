use std::env;

use cli_player::character_pallet;
use cli_player::video::Video;
use cli_player::video_player;

#[tokio::main]
async fn main() {
    const WIDTH: u32 = 100;

    let character_pallets =
        match character_pallet::parse_pallets_from_file("character-pallets.txt") {
            Ok(p) => p,
            Err(e) => panic!("Error while parseing: {e}"),
    };

    let pallet = &character_pallets[&"ascii".to_string()];

    let url = env::args().nth(1).unwrap();
    println!("Downloading...");
    let video = Video::build_from_url(&url).await.unwrap();

    println!("Preprocessing frames...");
    video.preprocess(&pallet, WIDTH, true);
    println!("Done!");

    video_player::play_video(video, &pallet, WIDTH, true).await.unwrap();
}
