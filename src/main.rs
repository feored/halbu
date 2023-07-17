use std::path::Path;

use crate::d2s::get_version;

mod d2s;

const CHARACTER_FILE: &str = "C:/Users/feord/Saved Games/Diablo II/WindowBird.d2s";

fn get_header_value(bytes: &Vec<u8>, id: d2s::HeaderID) -> Vec<u8> {
    let mut header_value: Vec<u8> = Vec::new();
    let (header_start, header_end) = d2s::get_header_bytes_range(id);
    for i in header_start..header_end {
        header_value.push(bytes[i]);
    }
    header_value
}

fn main() {
    let path: &Path = Path::new(CHARACTER_FILE);
    let save_file: Vec<u8> = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => panic!("File invalid: {e:?}"),
    };
    let valid_file: bool = d2s::check_valid_signature(&save_file);
    println!("This file is valid : {}", valid_file);

    let character_name: Vec<u8> = get_header_value(&save_file, d2s::HeaderID::CharacterName);
    let character_name_as_string = String::from_utf8(character_name);
    match character_name_as_string{
        Ok(c_name) => println!("Character name: '{c_name}'"),
        Err(e) => eprintln!("Error reading character name: {e:?}")
    }
    let mut version_bytes: [u8;4] = Default::default();
    version_bytes.copy_from_slice(&get_header_value(&save_file, d2s::HeaderID::VersionID)[0..4]);
    let version: Result<d2s::Version, &str> = d2s::get_version(&version_bytes);
    match version{
        Ok(version) => println!("Detected Version: '{version:?}'"),
        Err(e) => eprintln!("Error reading version: {e:?}")
    }

}
