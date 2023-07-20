pub const OFFSET : usize = 633;
const HEADER : [u8;8] = [0x57, 0x53, 0x01, 0x00, 0x00, 0x00, 0x50, 0x00];
const DIFFICULTY_HEADER : [u8;2] = [0x02, 0x01];
const DIFFICULTY_LENGTH : usize = 24;
const TRAILER : u8 = 0x01;


pub fn build_section() -> Vec<u8>{
    let mut waypoints_section = vec!();
    waypoints_section.extend_from_slice(&HEADER);
    for i in 0..3{
        waypoints_section.extend_from_slice(&DIFFICULTY_HEADER);
        waypoints_section.push(0x01);
        let mut remaining_bytes = DIFFICULTY_LENGTH - DIFFICULTY_HEADER.len() - 1;
        for j in 0..remaining_bytes{
            waypoints_section.push(0x00);
        }
    }
    waypoints_section.push(TRAILER);
    waypoints_section
}