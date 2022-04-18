use crate::utils;

use super::block::{Block, DataBlock};
use super::block_header::*;
use super::mdf4_enums::ZipType;

use miniz_oxide::inflate::decompress_to_vec;

#[derive(Debug, Clone, PartialEq)]
pub struct Dzblock {
    header: BlockHeader,

    dz_org_block_type: [u8; 2],

    dz_zip_type: ZipType,
    dz_reserved: u8,

    dz_zip_parameter: u32,

    dz_org_data_length: u64,

    dz_data_length: u64,

    dz_data: Vec<u8>,
}

impl DataBlock for Dzblock {
    fn data_array(&self, _stream: &[u8], _little_endian: bool) -> Vec<u8> {
        let decompressed =
            decompress_to_vec(self.dz_data.as_slice()).expect("Failed to decompress!");
        decompressed
    }
}

impl Block for Dzblock {
    fn new() -> Self {
        Self {
            header: BlockHeader::create("##DZ", 50, 0),
            dz_org_block_type: [0_u8; 2],
            dz_zip_type: ZipType::Deflate,
            dz_reserved: 0_u8,
            dz_zip_parameter: 0_u32,
            dz_org_data_length: 0_u64,
            dz_data_length: 0_u64,
            dz_data: Vec::new(),
        }
    }
    fn default() -> Self {
        Self {
            header: BlockHeader::create("##DZ", 50, 0),
            dz_org_block_type: [0_u8; 2],
            dz_zip_type: ZipType::Deflate,
            dz_reserved: 0_u8,
            dz_zip_parameter: 0_u32,
            dz_org_data_length: 0_u64,
            dz_data_length: 0_u64,
            dz_data: Vec::new(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (mut pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##DZ".as_bytes()) {
            panic!("Error DZBLOCK");
        }

        let dz_org_block_type = utils::read(stream, little_endian, &mut pos);
        let dz_zip_type = ZipType::new(utils::read(stream, little_endian, &mut pos));
        let dz_reserved = utils::read(stream, little_endian, &mut pos);
        let dz_zip_parameter = utils::read(stream, little_endian, &mut pos);
        let dz_org_data_length = utils::read(stream, little_endian, &mut pos);
        let dz_data_length = utils::read(stream, little_endian, &mut pos);
        let dz_data = stream[pos..pos + dz_data_length as usize].to_vec();

        pos += dz_data.len();

        (
            pos,
            Self {
                header,
                dz_org_block_type,
                dz_zip_type,
                dz_reserved,
                dz_zip_parameter,
                dz_org_data_length,
                dz_data_length,
                dz_data,
            },
        )
    }

    fn byte_len(&self) -> usize {
        todo!()
    }
}
