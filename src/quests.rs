pub const OFFSET : usize = 335;
const HEADER: [u8; 10] = [0x57, 0x6F, 0x6F, 0x21, 0x06, 0x00, 0x00, 0x00, 0x2A, 0x01];
// Woo! + unknown
const SECTION_LENGTH: usize = 298;

pub struct QuestSet {}

pub struct QuestFlags {
    pub warriv_introduction: u16,
    pub warriv_travel: u16,
    pub jerhyn_introduction: u16,
}

pub fn build_section() -> Vec<u8> {
    let mut section = vec!();
    section.extend_from_slice(&HEADER);
    for i in 0..(SECTION_LENGTH - HEADER.len()){
        section.push(0x00);
    }
    section
}
