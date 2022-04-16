use crate::utils;

use super::block::Block;
use super::block_header::*;
use super::mdf4::link_extract;

#[derive(Debug, Clone, PartialEq)]
struct Srblock {
    header: BlockHeader,

    sr_sr_next: u64,

    sr_data: u64,

    sr_cycle_count: u64,

    sr_interval: f64,

    sr_sync_type: u8,

    sr_flags: u8,
    sr_reserved: [u8; 6],
}

impl Block for Srblock {
    fn new() -> Self {
        Self {
            header: BlockHeader::create("##SR", 50, 0),
            sr_sr_next: 0_u64,
            sr_data: 0_u64,
            sr_cycle_count: 0_u64,
            sr_interval: 0_f64,
            sr_sync_type: 0_u8,
            sr_flags: 0_u8,
            sr_reserved: [0_u8; 6],
        }
    }
    fn default() -> Self {
        Self {
            header: BlockHeader::create("##SR", 50, 0),
            sr_sr_next: 0_u64,
            sr_data: 0_u64,
            sr_cycle_count: 0_u64,
            sr_interval: 0_f64,
            sr_sync_type: 0_u8,
            sr_flags: 0_u8,
            sr_reserved: [0_u8; 6],
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##SR".as_bytes()) {
            panic!("Error SRBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let sr_sr_next = address.remove(0);
        let sr_data = address.remove(0);

        let sr_cycle_count = utils::read(stream, little_endian, &mut pos);
        let sr_interval = utils::read(stream, little_endian, &mut pos);
        let sr_sync_type = utils::read(stream, little_endian, &mut pos);
        let sr_flags = utils::read(stream, little_endian, &mut pos);
        let sr_reserved: [u8; 6] = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                header,
                sr_sr_next,
                sr_data,
                sr_cycle_count,
                sr_interval,
                sr_sync_type,
                sr_flags,
                sr_reserved,
            },
        )
    }

    fn byte_len(&self) -> usize {
        todo!()
    }
}
