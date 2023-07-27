const SECTION_HEADER : [u8;2] = [0x4A, 0x4D];

const NO_ITEMS : [u8;13] = [0x4A, 0x4D, 0x00, 0x00, 0x4A, 0x4D, 0x00, 0x00, 0x6A, 0x66, 0x6B, 0x66, 0x00];

#[derive(PartialEq, Eq, Debug, Default)]
pub struct Placeholder {
    data: Vec<u8>,
}

pub fn parse(byte_vector: &Vec<u8>) -> Placeholder {
    let mut placeholder: Placeholder = Placeholder {
        data: Vec::<u8>::new(),
    };
    placeholder.data = byte_vector.clone();

    placeholder
}

pub fn generate(_placeholder: &Placeholder) -> Vec<u8> {
    let byte_vector: Vec<u8> = NO_ITEMS.to_vec();
    byte_vector
}
