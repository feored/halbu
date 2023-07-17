use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

mod d2s;



#[allow(dead_code)]
const OFFSET_CHECKSUM : usize = 12;
const OFFSET_MAP_SEED : usize = 171;
#[allow(dead_code)]
const CHARACTER_FILE : &str = "C:/Users/feord/Saved Games/Diablo II/WindowBird.d2s";


fn check_valid(bytes : &Vec<u8>) -> bool {
    bytes[0..4] == d2s::D2S_SIGNATURE
}

fn get_map_seed(bytes : &Vec<u8>) -> [u8; 4]{
    let mut map_seed : [u8;4] = [0; 4];
    for i in 0usize..4usize{
        map_seed[i] = bytes[OFFSET_MAP_SEED + i];
    }
    map_seed
}

fn get_header_value(bytes: &Vec<u8>, id : d2s::HeaderID) -> Vec<u8> {
    let mut header_value : Vec<u8> = Vec::new();
    let header_data: d2s::HeaderSection = d2s::get_header_data(id);
    for i in header_data.offset..(header_data.offset+header_data.bytes){
        header_value.push(bytes[i]);
    }
    header_value
}


fn main() {
    let path: &Path = Path::new(CHARACTER_FILE);
    let save_file: Vec<u8> = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => panic!("File invalid")
    };
    let valid_file: bool = check_valid(&save_file);
    println!("This file is valid : {}", valid_file);

    println!("Map seed: {:X?}", get_map_seed(&save_file));

    let character_name: Vec<u8> = get_header_value(&save_file, d2s::HeaderID::CharacterName);
    if let Ok(character_name_as_string) = String::from_utf8(character_name) {
        println!("Character name: '{}'", character_name_as_string);
    } else {
        println!("Invalid character name.");
    }
}
