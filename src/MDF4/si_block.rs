use crate::utils;

use super::block::Block;
use super::block_header::*;
use super::mdf4::link_extract;
use super::mdf4_enums::{BusType, SourceType};

#[derive(Debug, Clone)]
pub struct Siblock {
    #[allow(dead_code)]
    si_tx_name: u64,
    #[allow(dead_code)]
    si_tx_path: u64,
    #[allow(dead_code)]
    si_md_comment: u64,
    #[allow(dead_code)]
    si_type: SourceType,
    #[allow(dead_code)]
    si_bus_type: BusType,
    #[allow(dead_code)]
    si_flags: u8,
}
impl Block for Siblock {
    fn new() -> Self {
        Siblock {
            si_tx_name: 0_u64,
            si_tx_path: 0_u64,
            si_md_comment: 0_u64,
            si_type: SourceType::Bus,
            si_bus_type: BusType::Can,
            si_flags: 0_u8,
        }
    }
    fn default() -> Self {
        Siblock {
            si_tx_name: 0_u64,
            si_tx_path: 0_u64,
            si_md_comment: 0_u64,
            si_type: SourceType::Bus,
            si_bus_type: BusType::Can,
            si_flags: 0_u8,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##SI".as_bytes()) {
            panic!("Error SIBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let si_tx_name = address.remove(0);
        let si_tx_path = address.remove(0);
        let si_md_comment = address.remove(0);

        let si_type = SourceType::new(utils::read(stream, little_endian, &mut pos));
        let si_bus_type = BusType::new(utils::read(stream, little_endian, &mut pos));
        let si_flags = utils::read(stream, little_endian, &mut pos);

        let _si_reserved: [u8; 5] = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Siblock {
                si_tx_name,
                si_tx_path,
                si_md_comment,
                si_type,
                si_bus_type,
                si_flags,
            },
        )
    }

    fn byte_len(&self) -> usize {
        todo!()
    }
}

#[test]
fn si_read_test() {}
