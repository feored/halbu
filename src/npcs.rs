pub const OFFSET: usize = 714;
const HEADER: [u8; 3] = [0x77, 0x34, 0x00];
const NPCS_LENGTH: usize = 40;

pub fn build_section() -> Vec<u8> {
    let mut section = vec![];
    section.extend_from_slice(&HEADER);
    for _i in 0..NPCS_LENGTH {
        section.push(0x00);
    }
    section
}
