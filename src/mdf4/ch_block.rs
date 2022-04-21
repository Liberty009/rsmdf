use std::mem;

use crate::utils;

use super::block::Block;
use super::block_header::*;
use super::mdf4_enums::ChannelHierarchyType;
use super::mdf4_file::link_extract;

pub struct Chblock {
    header: BlockHeader,
    ch_ch_next: u64,
    ch_ch_first: u64,
    ch_tx_name: u64,
    ch_md_comment: u64,
    ch_element: Vec<u64>,
    ch_element_count: u32,
    ch_type: ChannelHierarchyType,
}
impl Block for Chblock {
    fn new() -> Self {
        Self {
            header: BlockHeader::create("##CH", 50, 0),
            ch_ch_next: 0_u64,
            ch_ch_first: 0_u64,
            ch_tx_name: 0_u64,
            ch_md_comment: 0_u64,
            ch_element: Vec::new(),
            ch_element_count: 0_u32,
            ch_type: ChannelHierarchyType::Function,
        }
    }
    fn default() -> Self {
        Self {
            header: BlockHeader::create("##CH", 50, 0),
            ch_ch_next: 0_u64,
            ch_ch_first: 0_u64,
            ch_tx_name: 0_u64,
            ch_md_comment: 0_u64,
            ch_element: Vec::new(),
            ch_element_count: 0_u32,
            ch_type: ChannelHierarchyType::Function,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##CH".as_bytes()) {
            panic!("Error CHBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let ch_element_count = utils::read(stream, little_endian, &mut pos);
        let ch_type = ChannelHierarchyType::new(utils::read(stream, little_endian, &mut pos));

        let ch_ch_next = address.remove(0);
        let ch_ch_first = address.remove(0);
        let ch_tx_name = address.remove(0);
        let ch_md_comment = address.remove(0);
        let mut ch_element = Vec::with_capacity(ch_element_count as usize * 3);
        for _i in 0..(ch_element_count * 3) {
            ch_element.push(address.remove(0));
        }

        (
            pos,
            Self {
                header,
                ch_ch_next,
                ch_ch_first,
                ch_tx_name,
                ch_md_comment,
                ch_element,
                ch_element_count,
                ch_type,
            },
        )
    }

    fn byte_len(&self) -> usize {
        let mut length = self.header.byte_len()
            + mem::size_of_val(&self.ch_ch_next)
            + mem::size_of_val(&self.ch_ch_first)
            + mem::size_of_val(&self.ch_tx_name)
            + mem::size_of_val(&self.ch_md_comment)
            + mem::size_of_val(&self.ch_element_count)
            + mem::size_of_val(&self.ch_type);

        if !self.ch_element.is_empty() {
            length += mem::size_of_val(&self.ch_element[0]) * self.ch_element.len();
        }
        length
    }
}
