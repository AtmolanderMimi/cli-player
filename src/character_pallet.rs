use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::io;

#[derive(Debug)]
pub enum CharacterPalletParsingError {
    IoError(io::Error),
    FormattingError,
}

impl Display for CharacterPalletParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharacterPalletParsingError::IoError(e) => write!(f, "{}", e),
            CharacterPalletParsingError::FormattingError => write!(
                f,
                "There was an error in the formatting of the file being parsed",
            ),
        }
    }
}

impl Error for CharacterPalletParsingError {}

/// Stores a pallet of characters of differing luminosity values
#[derive(Clone)]
pub struct CharacterPallet {
    pub name: String,
    characters: Vec<char>, // Emptiest to densest
}

impl CharacterPallet {
    fn new(name: String, characters: Vec<char>) -> CharacterPallet {
        CharacterPallet { name, characters }
    }
}

impl CharacterPallet {
    /// Gives a character that conresponds best to the luminosity within the pallet
    /// 
    /// This assumes that the luminosity of the characters linear
    pub fn character_for_luminosity(&self, luminosity: u8) -> Option<char> {
        let nb_divisions = (self.characters.len() as u8).checked_sub(1)?;

        let luminosity_slice_width = u8::MAX as f32 / nb_divisions as f32;
        // Rust automitically rounds up
        let luminosity_slice_index = luminosity as f32 / luminosity_slice_width;

        Some(self.characters[luminosity_slice_index as usize])
    }
}

/// Parses `CharacterPallet` from a file, please refer to the formatting in `character-pallets.txt`
/// to understand how to properly format a file for parseing
pub fn parse_pallets_from_file(
    path: &str,
) -> Result<HashMap<String, CharacterPallet>, CharacterPalletParsingError> {
    let input = match fs::read_to_string(path) {
        Ok(i) => i,
        Err(e) => return Err(CharacterPalletParsingError::IoError(e)),
    };

    let mut character_pallets = HashMap::new();
    for (i, line) in input.lines().enumerate() {
        if line.ends_with(':') {
            // This SHOULD never cause a crash
            let name = line[0..(line.len() - 1)].to_string();
            let mut characters = match input.lines().nth(i + 1) {
                Some(l) => l.chars().collect::<Vec<char>>(),
                None => return Err(CharacterPalletParsingError::FormattingError),
            };
            // Reverts the character to be emptiest to densest
            characters.reverse();

            let new_character_pallet = CharacterPallet::new(name.clone(), characters);
            character_pallets.insert(name, new_character_pallet);
        }
    }

    Ok(character_pallets)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_ascii_pallet() -> CharacterPallet {
        let character = " `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@"
            .chars().collect();
        CharacterPallet::new("ascii".to_string(), character)
    }

    /// This test assumes that there is a valid `character-pallets.txt` file,
    /// that it is well formatted and contains atleast one pallet
    #[test]
    fn parseing_works() {
        let pallets =
            parse_pallets_from_file("character-pallets.txt").unwrap();

        assert_ne!(0, pallets.len())
    }

    #[test]
    fn character_for_luminosity_works() {
        let pallet = new_ascii_pallet();
        let empty_pallet = CharacterPallet::new("pallet".to_string(), Vec::new());

        assert_eq!(Some('@'), pallet.character_for_luminosity(255));
        assert_eq!(Some(':'), pallet.character_for_luminosity(15));
        assert_eq!(Some(' '), pallet.character_for_luminosity(0));
        assert_eq!(None, empty_pallet.character_for_luminosity(141))
    }
}
