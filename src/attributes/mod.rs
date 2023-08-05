use std::fmt;

use serde::{Deserialize, Serialize};

use crate::utils::BytePosition;
use crate::utils::write_bits;
use crate::utils::read_bits;
use crate::data::read_csv;
use crate::ParseError;

mod tests;

const SECTION_HEADER: [u8; 2] = [0x67, 0x66];
const SECTION_TRAILER: u32 = 0x1FF;
const STAT_HEADER_LENGTH: usize = 9;
const STAT_NUMBER : usize = 16;
const DATA_PATH : &'static str = "assets/data/itemstatcost.txt";


/// Representation of a single stat, with data taken from itemstatcosts.txt
#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Stat{
    pub id: u32,
    pub name: String,
    pub bit_length: usize,
    pub value: u32
}

impl Stat {
    pub fn max(&self) -> u32 {
        (2u64.pow(self.bit_length as u32) - 1) as u32
    }
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{0} - {1}: {2} -- {3}bits [0-{4}])", self.id, self.name, self.value, self.bit_length, self.max())
    }
}


/// Representation of a character's attributes.
///
/// Certain values are fixed point and stored with integer and
/// fraction separately for precision and easier comparison.
#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Attributes {
    pub stats: Vec<Stat>
}

impl fmt::Display for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut final_string = String::default();
        for i in 0..self.stats.len(){
            final_string.push_str(&format!("{0}\n", self.stats[i]));
        }
        write!(f, "{0}", final_string)
    }
}

impl Attributes {

    /// Get a byte-aligned vector of bytes representing a character's attribute.
    pub fn write(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::<u8>::new();
        let mut byte_position: BytePosition = BytePosition::default();
        result.append(&mut SECTION_HEADER.to_vec());
        byte_position.current_byte += 2;
        for i in 0..self.stats.len(){
            write_bits(&mut result, &mut byte_position, self.stats[i].id, STAT_HEADER_LENGTH);
            write_bits(&mut result, &mut byte_position, self.stats[i].value, self.stats[i].bit_length);
        }
        write_bits(&mut result, &mut byte_position, SECTION_TRAILER, STAT_HEADER_LENGTH);
        result
    }
}


    /// Parse vector of bytes containing attributes data while storing byte position and return an Attributes struct.
    ///
    /// This function borrows a byte_position, which will store the length in bytes of the
    /// attributes section to help find the offset at which to start reading the next section.
    ///
    /// Attributes are stored in a pair format (header:value). Not all attributes are required to be
    /// present. Headers are always 9 bits. Values span different number of bits stored in CSV_BITLENGTH
    pub fn parse(
        byte_vector: &Vec<u8>,
        byte_position: &mut BytePosition,
    ) -> Result<Attributes, ParseError> {
        if byte_vector[0..2] != SECTION_HEADER {
            return Err(ParseError {
                message: format!(
                    "Found wrong header for attributes, expected {0:X?} but found {1:X?}",
                    SECTION_HEADER,
                    &byte_vector[0..2]
                ),
            });
        }
        byte_position.current_byte = 2;
        let mut attributes = Attributes::default();
        let csv_data: Vec<std::collections::HashMap<String, String>> = match read_csv(String::from(DATA_PATH)){
            Ok(res) => res,
            Err(e) => return Err(ParseError{message: e.to_string()})
        };

        // initialize all so that we always return all 16 stats
        for i in 0..STAT_NUMBER {
            let id = u32::from_str_radix(&csv_data[i]["*ID"], 10).unwrap();
            let stat_bit_length = usize::from_str_radix(&csv_data[i]["CSvBits"], 10).unwrap();
            attributes.stats.push(Stat { id: id, name: csv_data[i]["Stat"].clone(), bit_length: stat_bit_length, value: 0 });
        }
        // In case all stats are written down, parse one more to make sure we parse 0x1FF trailer
        for _i in 0..(STAT_NUMBER+1) {
            let header: u32 = read_bits(byte_vector, byte_position, STAT_HEADER_LENGTH);
            if header == SECTION_TRAILER {
                break;
            }
            for j in 0..STAT_NUMBER {
                if attributes.stats[j].id == header{
                    let value = read_bits(byte_vector, byte_position, attributes.stats[j].bit_length);
                    attributes.stats[j].value = value;
                }
            }
        }
        Ok(attributes)
    }