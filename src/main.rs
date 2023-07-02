use yt_terminal::character_pallet;

fn main() {
    let character_pallets =
        match character_pallet::parse_pallets_from_file("character-pallets.txt") {
            Ok(p) => p,
            Err(e) => panic!("Error while parseing: {e}"),
        };
}
