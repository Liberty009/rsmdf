use crate::utils;

use super::block::Block;
use super::block_header::*;
use super::mdf4::link_extract;
use super::mdf4_enums::{SourceType, BusType};

#[derive(Debug, Clone, PartialEq)]
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
    #[allow(dead_code)]
    si_reserved: [u8; 5],
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
            si_reserved: [0_u8; 5],
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
            si_reserved: [0_u8; 5],
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

        let si_reserved = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Siblock {
                si_tx_name,
                si_tx_path,
                si_md_comment,
                si_type,
                si_bus_type,
                si_flags,
                si_reserved,
            },
        )
    }

    fn byte_len(&self) -> usize {
        todo!()
    }
}

#[test]
fn si_read_test() {

let raw: [u8; 56] = [
	0x23, 0x23, 0x53, 0x49, 0x00, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0xF0, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x45, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
];

let (pos, si) = Siblock::read(&raw, 0, true);

assert_eq!(pos, 56);
assert_eq!(17648, si.si_tx_name);
assert_eq!(17680, si.si_tx_path);
assert_eq!(0, si.si_md_comment);
assert_eq!(SourceType::Ecu, si.si_type);
assert_eq!(BusType::Other, si.si_bus_type);
assert_eq!(0, si.si_flags);
assert!(utils::eq(&si.si_reserved, &[0_u8; 5]));

}
