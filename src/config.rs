use std::fmt::Display;

use clap::Parser;

use crate::character_pallet::{CharacterPallet, self};

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

    /// Limits the frame rate to this
    #[arg(short, long, default_value_t = 30)]
    frame_limit: u32,

    /// Sets the volume (can be over 1.0)
    #[arg(short, long, default_value_t = 1.0)]
    volume: f32,

    /// Removes frame preprocessing (takes a lot VRAM)
    #[arg(long, default_value_t = false)]
    no_preprocess: bool,

    /// Disables the use of color
    #[arg(long, default_value_t = false)]
    no_color: bool,
}

pub struct PalletDoesNotExistError;

impl Display for PalletDoesNotExistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The pallet specified is not in the list of available pallets")
    }
}

pub struct Config {
    query: String,
    pallet: CharacterPallet,
    width: u32,
    frame_limit: u32,
    volume: f32,
    color: bool,
    preprocessing: bool, 
}

impl Config {
    fn build(query: String, pallet: String, width: u32, frame_limit: u32, volume: f32, color: bool, preprocessing: bool) -> Result<Config, PalletDoesNotExistError> {
        let character_pallets = match character_pallet::parse_pallets_from_file("character-pallets.txt") {
            Ok(p) => p,
            Err(e) => panic!("Error while parseing: {e}"),
        };

        let pallet = match character_pallets.get(&pallet) {
            Some(p) => p,
            None => return Err(PalletDoesNotExistError),
        };
        let pallet = (*pallet).clone();

        let config = Config {
            query,
            pallet,
            width,
            frame_limit,
            volume,
            color,
            preprocessing,
        };

        Ok(config)
    }

    pub fn build_from_args() -> Result<Config, PalletDoesNotExistError> {
        let args = Args::parse();

        let config = Config::build(
            args.url_or_path,
            args.pallet,
            args.width,
            args.frame_limit,
            args.volume,
            !args.no_color,
            !args.no_preprocess,
        )?;

        Ok(config)
    }
}

impl Config {
    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn pallet(&self) -> &CharacterPallet {
        &self.pallet
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn frame_limit(&self) -> u32 {
        self.frame_limit
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn color(&self) -> bool {
        self.color
    }

    pub fn preprocessing(&self) -> bool {
        self.preprocessing
    }
}