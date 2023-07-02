use std::collections::HashMap;
use std::error::Error;
use std::fmt::write;
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
pub struct CharacterPallet {
    pub name: String,
    characters: Vec<char>,
}

impl CharacterPallet {
    fn new(name: String, characters: Vec<char>) -> CharacterPallet {
        CharacterPallet { name, characters }
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
            let characters = match input.lines().nth(i + 1) {
                Some(l) => l.chars().collect::<Vec<char>>(),
                None => return Err(CharacterPalletParsingError::FormattingError),
            };

            let new_character_pallet = CharacterPallet::new(name.clone(), characters);
            character_pallets.insert(name, new_character_pallet);
        }
    }

    Ok(character_pallets)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This test assumes that there is a valid `character-pallets.txt` file,
    /// that it is well formatted and contains atleast one pallet
    #[test]
    fn parseing_works() {
        let pallets =
            parse_pallets_from_file("character-pallets.txt").unwrap();

        assert_ne!(0, pallets.len())
    }
}
