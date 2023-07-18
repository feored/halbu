const D2S_SIGNATURE: [u8; 4] = [0x55, 0xAA, 0x55, 0xAA];

struct HeaderSection {
    pub offset: usize,
    pub bytes: usize,
}
#[derive(Debug)]
pub enum HeaderID {
    Signature,
    VersionID,
    FileSize,
    Checksum,
    ActiveWeapon,
    CharacterName,
    CharacterStatus,
    CharacterProgression,
}

pub fn get_header_bytes_range(id: HeaderID) -> (usize, usize) {
    let header_data: HeaderSection = get_header_data(id);
    (header_data.offset, header_data.offset + header_data.bytes)
}

fn get_header_data(id: HeaderID) -> HeaderSection {
    match id {
        HeaderID::Signature => HeaderSection {
            offset: (0),
            bytes: (4),
        },
        HeaderID::VersionID => HeaderSection {
            offset: (4),
            bytes: (4),
        },
        HeaderID::FileSize => HeaderSection {
            offset: (8),
            bytes: (4),
        },
        HeaderID::Checksum => HeaderSection {
            offset: (12),
            bytes: (4),
        },
        HeaderID::ActiveWeapon => HeaderSection {
            offset: (16),
            bytes: (4),
        },
        HeaderID::CharacterName => HeaderSection {
            offset: (20),
            bytes: (16),
        },
        HeaderID::CharacterStatus => HeaderSection {
            offset: (36),
            bytes: (1),
        },
        HeaderID::CharacterProgression => HeaderSection {
            offset: (37),
            bytes: (1),
        },
    }
}

#[derive(Debug)]
pub enum Version {
    v100,
    v107,
    v108,
    v109,
    v110,
}

pub fn get_version(version_bytes: &[u8; 4]) -> Result<Version, &'static str> {
    let version_number: u32 = u32::from_le_bytes(*version_bytes);
    match version_number {
        71 => Ok(Version::v100),
        87 => Ok(Version::v107),
        89 => Ok(Version::v108),
        92 => Ok(Version::v109),
        96 => Ok(Version::v110),
        _ => Err("version ID does not match any known version of the game."),
    }
}

pub fn check_valid_signature(bytes: &Vec<u8>) -> bool {
    let (header_start, header_end) = get_header_bytes_range(HeaderID::Signature);
    bytes[header_start..header_end] == D2S_SIGNATURE
}

// checksum = 0
//     for i in range(len(data)):
//         ch = data[i]
//         if (i >= OFFSET_CHECKSUM_START and i < OFFSET_CHECKSUM_END):
//             ch = 0
//         checksum = ctypes.c_int32((checksum << 1) + ch + ctypes.c_int32(checksum < 0).value).value
//     return checksum

pub fn calc_checksum(bytes: &Vec<u8>) -> i32{
    let mut checksum : i32 = 0;
    let (checksum_start, checksum_end) = get_header_bytes_range(HeaderID::Checksum);
    for i in 0..bytes.len(){
        let mut ch : i32 = bytes[i] as i32;
        if i >= checksum_start && i < checksum_end {
            ch = 0;
        }
        checksum = (checksum << 1) + ch + ((checksum < 0) as i32);
    }
    checksum
}