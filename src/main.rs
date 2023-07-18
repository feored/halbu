use std::error::Error;
use std::io::Write;
use std::path::Path;
use std::fs::OpenOptions;
use std::cmp;

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


fn save_file(path : &Path, data : Vec<u8>) -> std::io::Result<()>{
    let mut file: std::fs::File = OpenOptions::new().read(true).write(true).open(path)?;
    file.write_all(&data)
}

fn main() {
    let path: &Path = Path::new(CHARACTER_FILE);
    let mut file_contents: Vec<u8> = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => panic!("File invalid: {e:?}")
    };
    let valid_file: bool = d2s::check_valid_signature(&file_contents);
    println!("This file is valid : {}", valid_file);

    let character_name: Vec<u8> = get_header_value(&file_contents, d2s::HeaderID::CharacterName);
    let character_name_as_string = String::from_utf8(character_name);
    match character_name_as_string {
        Ok(c_name) => println!("Character name: '{c_name}'"),
        Err(e) => eprintln!("Error reading character name: {e:?}"),
    }
    

    // // Change name
    // let (name_start, name_end) = d2s::get_header_bytes_range(d2s::HeaderID::CharacterName);
    // let mut new_name_bytes: [u8;16] = [0; 16]; 
    // let new_name = "George".as_bytes();
    // for i in 0..new_name.len(){
    //     new_name_bytes[i] = new_name[i];
    // }
    // for i in name_start..name_end{
    //     file_contents[i] = new_name_bytes[i - name_start];
    // }
    // // Note: if file name != character name, file won't open


    let mut version_bytes: [u8; 4] = Default::default();
    version_bytes.copy_from_slice(&get_header_value(&file_contents, d2s::HeaderID::VersionID)[0..4]);
    let version: Result<d2s::Version, &str> = d2s::get_version(&version_bytes);
    match version {
        Ok(version) => println!("Detected Version: '{version:?}'"),
        Err(e) => eprintln!("Error reading version: {e:?}"),
    }

    let checksum_bytes: [u8; 4] = d2s::calc_checksum(&file_contents).to_le_bytes();
    println!("Checksum: {:X?}", checksum_bytes);



    // replace checksum
    let (checksum_start, checksum_end) = d2s::get_header_bytes_range(d2s::HeaderID::Checksum);
    for i in checksum_start..checksum_end{
        file_contents[i] = checksum_bytes[i - checksum_start];
    }
    match save_file(path, file_contents){
        Ok(()) => println!("Successfully saved file: {path:?}"),
        Err(e) => eprintln!("Error while saving file : {e:?}")
    }

}
