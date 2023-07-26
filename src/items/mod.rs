#[derive(PartialEq, Eq, Debug, Default)]
pub struct Placeholder {
    data: Vec<u8>,
}

pub fn parse(byte_vector: &mut Vec<u8>) -> Placeholder {
    let mut placeholder: Placeholder = Placeholder {
        data: Vec::<u8>::new(),
    };
    placeholder.data.append(byte_vector);

    placeholder
}

pub fn generate(placeholder: &mut Placeholder) -> Vec<u8> {
    let mut byte_vector: Vec<u8> = Vec::<u8>::new();
    byte_vector.append(&mut placeholder.data);

    byte_vector
}
