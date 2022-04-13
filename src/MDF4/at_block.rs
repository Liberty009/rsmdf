use crate::utils;

use super::block::Block;
use super::block_header::BlockHeader;
use super::mdf4::link_extract;

#[derive(Debug, Clone, PartialEq)]
pub struct Atblock {
    //id: [u8; 4],
    //reserved0: [u8; 4],
    //block_len: u64,
    //links_nr: u64,
    #[allow(dead_code)]
    next_at_addr: u64,
    #[allow(dead_code)]
    file_name_addr: u64,
    #[allow(dead_code)]
    mime_addr: u64,
    #[allow(dead_code)]
    comment_addr: u64,
    #[allow(dead_code)]
    flags: u16,
    #[allow(dead_code)]
    creator_index: u16,
    //reserved1: [u8; 4],
    #[allow(dead_code)]
    md5_sum: [u8; 16],
    #[allow(dead_code)]
    original_size: u64,
    #[allow(dead_code)]
    embedded_size: u64,
    #[allow(dead_code)]
    embedded_data: Vec<u8>,
}

impl Block for Atblock {
    fn new() -> Self {
        Self {
            next_at_addr: 0,
            file_name_addr: 0,
            mime_addr: 0,
            comment_addr: 0,
            flags: 0,
            creator_index: 0,
            //reserved1: [0; 4],
            md5_sum: [0; 16],
            original_size: 0,
            embedded_size: 0,
            embedded_data: Vec::new(),
        }
    }
    fn default() -> Self {
        Self {
            next_at_addr: 0,
            file_name_addr: 0,
            mime_addr: 0,
            comment_addr: 0,
            flags: 0,
            creator_index: 0,
            //reserved1: [0; 4],
            md5_sum: [0; 16],
            original_size: 0,
            embedded_size: 0,
            embedded_data: Vec::new(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##AT".as_bytes()) {
            panic!("Error: block id doesn't match Attachment Block");
        }

        let (mut pos, addresses) = link_extract(stream, pos, little_endian, header.link_count);

        let next_at_addr = addresses[0];
        let file_name_addr = addresses[1];
        let mime_addr = addresses[2];
        let comment_addr = addresses[3];

        let flags = utils::read(stream, little_endian, &mut pos);
        let creator_index = utils::read(stream, little_endian, &mut pos);
        let _reserved1: [u8; 4] = utils::read(stream, little_endian, &mut pos);
        let md5_sum = utils::read(stream, little_endian, &mut pos);
        let original_size = utils::read(stream, little_endian, &mut pos);
        let embedded_size = utils::read(stream, little_endian, &mut pos);
        let embedded_data = stream[pos..pos + embedded_size as usize].to_vec();

        (
            pos,
            Self {
                //id: header.id,
                //reserved0: header.reserved0,
                //block_len: header.length,
                //links_nr: header.link_count,
                next_at_addr,
                file_name_addr,
                mime_addr,
                comment_addr,
                flags,
                creator_index,
                //reserved1,
                md5_sum,
                original_size,
                embedded_size,
                embedded_data,
            },
        )
    }

    fn byte_len(&self) -> usize {
        todo!()
    }
}
