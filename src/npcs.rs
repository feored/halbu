const SECTION_HEADER: [u8; 3] = [0x77, 0x34, 0x00];

#[derive(PartialEq, Eq, Debug)]
pub struct Placeholder {
    data: [u8;51]
}

impl Default for Placeholder {
    fn default() -> Self {
        Placeholder {data: [0x00;51]}
    }
}

pub fn parse(bytes: &[u8;51]) -> Placeholder{
    let mut placeholder: Placeholder = Placeholder {data: [0x00; 51]};
    placeholder.data.copy_from_slice(bytes);

    placeholder
}

pub fn generate(placeholder:Placeholder) -> [u8;51]{
    let mut bytes: [u8;51] = [0x00;51];
    bytes.copy_from_slice(&placeholder.data[0..51]);

    bytes
}