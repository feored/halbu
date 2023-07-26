const SECTION_HEADER: [u8; 3] = [0x77, 0x34, 0x00];

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct Placeholder {
    data: [u8;52]
}

pub fn parse(bytes: &[u8;52]) -> Placeholder{
    let mut placeholder: Placeholder = Placeholder {data: [0x00; 52]};
    placeholder.data.copy_from_slice(bytes);

    placeholder
}

pub fn generate(placeholder:Placeholder) -> [u8;52]{
    let mut bytes: [u8;52] = [0x00;52];
    bytes.copy_from_slice(&placeholder.data[0..52]);

    bytes
}