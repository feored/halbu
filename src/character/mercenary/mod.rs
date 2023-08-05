

use std::ops::Range;
use std::fmt;
use serde::{Deserialize, Serialize};
use crate::ParseError;

mod tests;
const RANGE_IS_DEAD : Range<usize> = 0..2;
const RANGE_ID : Range<usize> = 2..6;
const RANGE_NAME_ID: Range<usize> = 6..8;
const RANGE_VARIANT_ID: Range<usize> = 8..10;
const RANGE_EXPERIENCE : Range<usize> = 10..14;


#[derive(Default, PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Mercenary {
    pub is_dead: bool,
    pub id: u32,
    pub name_id: u16,
    pub variant_id: u16,
    pub experience: u32,
}

impl fmt::Display for Mercenary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Dead: {0}, ID: {1}, Name_ID: {2}, Variant: {3}, XP: {4}", self.is_dead, self.id, self.name_id, self.variant_id, self.experience)
    }
}


pub fn read(data: &[u8; 14]) -> Result<Mercenary, ParseError> {
    let mut mercenary: Mercenary = Mercenary::default();
    if  u16::from_le_bytes(<[u8; 2]>::try_from(&data[RANGE_IS_DEAD]).unwrap()) != 0 {
        mercenary.is_dead = true;
    }

    mercenary.id = u32::from_le_bytes(<[u8; 4]>::try_from(&data[RANGE_ID]).unwrap());
    mercenary.variant_id = u16::from_le_bytes(<[u8; 2]>::try_from(&data[RANGE_VARIANT_ID]).unwrap());
    mercenary.name_id = u16::from_le_bytes(<[u8; 2]>::try_from(&data[RANGE_NAME_ID]).unwrap());
    mercenary.experience = u32::from_le_bytes(<[u8; 4]>::try_from(&data[RANGE_EXPERIENCE]).unwrap());

    Ok(mercenary)
}

impl Mercenary {
    pub fn write(&self) -> [u8; 14] {
        let mut bytes: [u8; 14] = [0x00; 14];
        bytes[RANGE_IS_DEAD].copy_from_slice(match self.is_dead {
            true => &[0x01, 0x00],
            false => &[0x00, 0x00],
        });
    
        bytes[RANGE_ID].copy_from_slice(&self.id.to_le_bytes());
        bytes[RANGE_NAME_ID].copy_from_slice(&self.name_id.to_le_bytes());    
        bytes[RANGE_VARIANT_ID].copy_from_slice(&self.variant_id.to_le_bytes());
        bytes[RANGE_EXPERIENCE].copy_from_slice(&self.experience.to_le_bytes());
        bytes
    }
    
    pub fn is_hired(&self) -> bool {
        self.id != 0u32
    }
}

