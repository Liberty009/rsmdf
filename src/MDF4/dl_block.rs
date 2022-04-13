use crate::utils;

use super::block::Block;
use super::block_header::*;
use super::mdf4::link_extract;

#[derive(Debug, Clone)]
struct Dlblock {
    #[allow(dead_code)]
    dl_dl_next: u64,
    #[allow(dead_code)]
    dl_data: Vec<u64>,
    #[allow(dead_code)]
    dl_flags: u8,
    #[allow(dead_code)]
    dl_count: u32,
    #[allow(dead_code)]
    dl_equal_length: u64,
    #[allow(dead_code)]
    dl_offset: Vec<u64>,
}
impl Block for Dlblock {
    fn new() -> Self {
        Self {
            dl_dl_next: 0_u64,
            dl_data: Vec::new(),
            dl_flags: 0_u8,
            dl_count: 0_u32,
            dl_equal_length: 0_u64,
            dl_offset: Vec::new(),
        }
    }
    fn default() -> Self {
        Self {
            dl_dl_next: 0_u64,
            dl_data: Vec::new(),
            dl_flags: 0_u8,
            dl_count: 0_u32,
            dl_equal_length: 0_u64,
            dl_offset: Vec::new(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##DL".as_bytes()) {
            panic!("Error DBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let dl_flags = utils::read(stream, little_endian, &mut pos);
        let dl_count = utils::read(stream, little_endian, &mut pos);
        let dl_equal_length = utils::read(stream, little_endian, &mut pos);
        let mut dl_offset = Vec::new();
        for _i in 0..dl_count {
            dl_offset.push(utils::read(stream, little_endian, &mut pos));
        }

        let dl_dl_next = address.remove(0);
        let mut dl_data = Vec::new();
        for _i in 0..dl_count {
            dl_data.push(address.remove(0));
        }

        (
            pos,
            Self {
                dl_dl_next,
                dl_data,
                dl_flags,
                dl_count,
                dl_equal_length,
                dl_offset,
            },
        )
    }

    fn byte_len(&self) -> usize {
        todo!()
    }
}
