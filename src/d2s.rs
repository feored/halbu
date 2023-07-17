use std::collections::HashMap;

pub const D2S_SIGNATURE: [u8; 4] = [0x55, 0xAA, 0x55, 0xAA];

pub struct HeaderSection {
    pub offset: usize,
    pub bytes: usize,
}

pub enum HeaderID {
    Identifier,
    VersionID,
    FileSize,
    Checksum,
    ActiveWeapon,
    CharacterName,
    CharacterStatus,
    CharacterProgression,
}

pub fn get_header_data(id: HeaderID) -> HeaderSection {
    match id {
        HeaderID::Identifier => HeaderSection {
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
