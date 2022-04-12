use super::utils as mdf4_utils;
use super::Block::Block;
use super::BlockHeader::*;
use crate::utils;

#[derive(Debug, Clone)]
pub struct Txblock {
    tx_data: String,
}

impl Txblock {
    pub fn text(&self) -> String {
        self.clone().tx_data
    }
}

impl Block for Txblock {
    fn new() -> Self {
        Self {
            tx_data: String::new(),
        }
    }
    fn default() -> Self {
        Self {
            tx_data: String::new(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##TX".as_bytes()) {
            panic!("Error type incorrect");
        }

        let length = header.length as usize - header.byte_len();

        let tx_data = mdf4_utils::str_from_u8(&stream[pos..(pos + length)]);

        (pos + length, Self { tx_data })
    }

    fn byte_len(&self) -> usize {
        24 + &self.tx_data.len() + 1
    }
}

#[test]
fn tx_read_test() {
    let raw: [u8; 40] = [
        0x23, 0x23, 0x54, 0x58, 0x00, 0x00, 0x00, 0x00, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x45, 0x6E, 0x67, 0x69, 0x6E, 0x65,
        0x5F, 0x31, 0x00, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    ];

    let (pos, tx) = Txblock::read(&raw, 0, true);

    assert_eq!(33, pos);
    //println!("{}", tx.tx_data);
    assert!(tx.tx_data.eq("Engine_1"))
}
