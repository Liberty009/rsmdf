use std::mem;

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

        if !utils::eq(&header.id, "##MD".as_bytes()) {
            panic!("Error type incorrect");
        }

        let tx_data = mdf4_utils::str_from_u8(&stream[pos..(pos + header.length as usize - 10)]);

        (pos + header.length as usize, Self { tx_data })
    }

	fn byte_len(&self) -> usize {
		mem::size_of_val(&self.tx_data)
	}
}
