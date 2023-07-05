use std::env;

use clap::Parser;

use cli_player::character_pallet;
use cli_player::video::Video;
use cli_player::video_player;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The url or path to use when searching the video
    #[arg(short, long)]
    url_or_path: String,

    /// Pallet of characters
    #[arg(short, long, default_value = "ascii")] 
    pallet: String,

    /// Nb of characters in width
    #[arg(short, long, default_value_t = 100)]
    width: u32,

    /// Preprocess the frames
    #[arg(long, default_value_t = false)]
    no_preprocess: bool,

    /// Use of color
    #[arg(long, default_value_t = false)]
    no_color: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let character_pallets =
        match character_pallet::parse_pallets_from_file("character-pallets.txt") {
            Ok(p) => p,
            Err(e) => panic!("Error while parseing: {e}"),
    };

    let pallet = &character_pallets[&"ascii".to_string()];

    let video = match Video::build_from_path(&args.url_or_path) {
        Ok(v) => v,
        Err(_) => {
            println!("Downloading...");
            Video::build_from_url(&args.url_or_path).await.unwrap()
        },
    };

    if !args.no_preprocess {
        println!("Preprocessing frames...");
        video.preprocess(&pallet, args.width, !args.no_color);
        println!("Done!");
    }

    video_player::play_video(video, &pallet, args.width, !args.no_color).await.unwrap();
}
