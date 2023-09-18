use log::warn;

const REFERENCE_CODE: [&'static str; 37] = [
    "10",
    "11111011",
    "1111100",
    "001100",
    "1101101",
    "11111010",
    "00010110",
    "1101111",
    "01111",
    "000100",
    "01110",
    "11110",
    "0101",
    "01000",
    "110001",
    "110000",
    "010011",
    "11010",
    "00011",
    "1111110",
    "000101110",
    "010010",
    "11101",
    "01101",
    "001101",
    "1111111",
    "11001",
    "11011001",
    "11100",
    "0010",
    "01100",
    "00001",
    "1101110",
    "00000",
    "00111",
    "0001010",
    "11011000",
];
const REFERENCE_STR: [char; 37] = [
    ' ', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
    'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub(crate) struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    character: Option<char>,
}

pub(crate) fn encode_char(c: char) -> &'static str {
    for (index, cref) in REFERENCE_STR.iter().enumerate() {
        if *cref == c {
            return REFERENCE_CODE[index];
        }
    }
    warn!("Failed to encode char: {0}. Using default (space) instead.", c);
    REFERENCE_CODE[0]
}

impl Node {
    pub(crate) fn insert_code(&mut self, bits: &'static str, character: char) {
        let count = bits.chars().count();
        if count == 0 {
            self.character = Some(character);
            return;
        }
        let c = bits.chars().next().unwrap();
        let target_node = if c == '1' { &mut self.right } else { &mut self.left };

        match target_node {
            &mut Some(ref mut subnode) => subnode.insert_code(&bits[1..count], character),
            &mut None => {
                let mut new_node = Node::default();
                new_node.insert_code(&bits[1..count], character);
                let boxed_node = Some(Box::new(new_node));
                *target_node = boxed_node;
            }
        }
    }

    pub(crate) fn build_huffman_tree() -> Node {
        let mut node = Node::default();
        for (index, code) in REFERENCE_CODE.iter().enumerate() {
            node.insert_code(code, REFERENCE_STR[index]);
        }
        node
    }

    pub(crate) fn decode(&self, mut bits: String) -> Option<char> {
        let count = bits.chars().count();
        if count == 0 {
            return self.character;
        }

        let c = bits.remove(0);
        let target_node = if c == '1' { &self.right } else { &self.left };
        return target_node.as_ref().unwrap().decode(bits);
    }
}
