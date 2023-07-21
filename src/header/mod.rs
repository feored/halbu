pub mod character;
pub mod mercenary;

const SIGNATURE: [u8; 4] = [0x55, 0xAA, 0x55, 0xAA];

const VERSION_100: u32 = 71;
const VERSION_107: u32 = 87;
const VERSION_108: u32 = 89;
const VERSION_109: u32 = 92;
const VERSION_110: u32 = 96;

pub const OFFSET : usize = 0;

#[derive(Debug)]
pub enum HeaderID {
    Signature,
    VersionID,
    FileSize,
    Checksum,
    WeaponSet,
    CharacterName,
    CharacterStatus,
    CharacterProgression,
}

#[derive(Debug)]
pub enum Version {
    V100,
    V107,
    V108,
    V109,
    V110,
}


struct FileSection {
    offset: usize,
    bytes: usize,
}


fn get_file_bytes_range(id: HeaderID) -> (usize, usize) {
    let data: FileSection = get_file_data(id);
    (data.offset, data.offset + data.bytes)
}

fn get_file_data(id: HeaderID) -> FileSection {
    match id {
        HeaderID::Signature => FileSection {
            offset: (0),
            bytes: (4),
        },
        HeaderID::VersionID => FileSection {
            offset: (4),
            bytes: (4),
        },
        HeaderID::FileSize => FileSection {
            offset: (8),
            bytes: (4),
        },
        HeaderID::Checksum => FileSection {
            offset: (12),
            bytes: (4),
        },
        HeaderID::WeaponSet => FileSection {
            offset: (16),
            bytes: (4),
        },
        HeaderID::CharacterName => FileSection {
            offset: (20),
            bytes: (16),
        },
        HeaderID::CharacterStatus => FileSection {
            offset: (36),
            bytes: (1),
        },
        HeaderID::CharacterProgression => FileSection {
            offset: (37),
            bytes: (1),
        },
    }
}

//Refactor into ::from
pub fn into_version(version_bytes: &[u8; 4]) -> Result<Version, &'static str> {
    let version_number: u32 = u32::from_le_bytes(*version_bytes);
    match version_number {
        VERSION_100 => Ok(Version::V100),
        VERSION_107 => Ok(Version::V107),
        VERSION_108 => Ok(Version::V108),
        VERSION_109 => Ok(Version::V109),
        VERSION_110 => Ok(Version::V110),
        _ => Err("version ID does not match any known version of the game."),
    }
}

fn check_valid_signature(bytes: &Vec<u8>) -> bool {
    let (header_start, header_end) = get_file_bytes_range(HeaderID::Signature);
    bytes[header_start..header_end] == SIGNATURE
}

pub fn calc_checksum(bytes: &Vec<u8>) -> i32 {
    let mut checksum: i32 = 0;
    let (checksum_start, checksum_end) = get_file_bytes_range(HeaderID::Checksum);
    for i in 0..bytes.len() {
        let mut ch: i32 = bytes[i] as i32;
        if i >= checksum_start && i < checksum_end {
            ch = 0;
        }
        checksum = (checksum << 1) + ch + ((checksum < 0) as i32);
    }
    checksum
}
