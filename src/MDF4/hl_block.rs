use crate::utils;

use super::block::Block;
use super::block_header::*;
use super::mdf4::link_extract;
use super::mdf4_enums::ZipType;

#[derive(Debug, Clone)]
struct Hlblock {
    #[allow(dead_code)]
    hl_dl_first: u64,
    #[allow(dead_code)]
    hl_flags: u16,
    #[allow(dead_code)]
    hl_zip_type: ZipType,
    //hl_reserved: [u8; 5],
}
impl Block for Hlblock {
    fn new() -> Self {
        Self {
            hl_dl_first: 0_u64,
            hl_flags: 0_u16,
            hl_zip_type: ZipType::Deflate,
            //hl_reserved: [0_u8; 5]
        }
    }
    fn default() -> Self {
        Self {
            hl_dl_first: 0_u64,
            hl_flags: 0_u16,
            hl_zip_type: ZipType::Deflate,
            //hl_reserved: [0_u8; 5]
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##HL".as_bytes()) {
            panic!("Error HLBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let hl_dl_first = address.remove(0);
        let hl_flags = utils::read(stream, little_endian, &mut pos);
        let hl_zip_type = ZipType::new(utils::read(stream, little_endian, &mut pos));
        let _hl_reserved: [u8; 5] = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                hl_dl_first,
                hl_flags,
                hl_zip_type,
                //hl_reserved,
            },
        )
    }

    fn byte_len(&self) -> usize {
        todo!()
    }
}
