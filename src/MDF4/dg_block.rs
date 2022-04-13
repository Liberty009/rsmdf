use std::mem;

use super::block::{Block, LinkedBlock};
use super::block_header::*;
use super::cg_block::Cgblock;
use super::mdf4::link_extract;
use crate::utils;

#[derive(Debug, Clone, PartialEq)]
pub struct Dgblock {
    #[allow(dead_code)]
    dg_dg_next: u64,
    #[allow(dead_code)]
    dg_cg_first: u64,
    #[allow(dead_code)]
    dg_data: u64,
    #[allow(dead_code)]
    dg_md_comment: u64,
    #[allow(dead_code)]
    dg_rec_id_size: u8,
    dg_reserved: [u8; 7],
}

impl LinkedBlock for Dgblock {
    fn next(&self, stream: &[u8], little_endian: bool) -> Option<Self> {
        if self.dg_dg_next == 0 {
            None
        } else {
            let (_pos, block) = Self::read(stream, self.dg_dg_next as usize, little_endian);
            Some(block)
        }
    }
    fn list(&self, stream: &[u8], little_endian: bool) -> Vec<Self> {
        let mut all = Vec::new();

        let next = self.next(stream, little_endian);

        all.push(self.clone());
        match next {
            None => {}
            Some(block) => all.append(&mut block.list(stream, little_endian)),
        }

        all

        // let next_block = self;

        // all.push(self.clone());
        // loop {
        //     let next_block = next_block.next(stream, little_endian);

        //     match next_block {
        //         Some(block) => all.push(block.clone()),
        //         None => break,
        //     }
        // }

        // all
    }
}

impl Dgblock {
    pub fn first(&self, stream: &[u8], little_endian: bool) -> Cgblock {
        let (_, block) = Cgblock::read(stream, self.dg_cg_first as usize, little_endian);
        block
    }

    // pub fn read_all(stream: &[u8], position: usize, little_endian: bool) -> Vec<Self> {
    //     let mut all = Vec::new();
    //     let mut next_dg = position;

    //     while next_dg != 0 {
    //         let (_pos, dg_block) = Dgblock::read(stream, next_dg, little_endian);
    //         next_dg = dg_block.dg_dg_next as usize;
    //         all.push(dg_block);
    //     }

    //     all
    // }

    pub fn read_channel_groups(self, stream: &[u8], little_endian: bool) -> Vec<Cgblock> {
        let mut channel_grps = Vec::new();
        let next = self.first(stream, little_endian); //dg_cg_first as usize;
        channel_grps.push(next.clone());

        loop {
            let next = next.next(stream, little_endian);

            match next {
                Some(dg) => channel_grps.push(dg.clone()),
                None => break,
            }
        }
        channel_grps
    }
}

impl Block for Dgblock {
    fn new() -> Self {
        Self {
            dg_dg_next: 0_u64,
            dg_cg_first: 0_u64,
            dg_data: 0_u64,
            dg_md_comment: 0_u64,
            dg_rec_id_size: 0_u8,
            dg_reserved: [0_u8; 7],
        }
    }
    fn default() -> Self {
        Self {
            dg_dg_next: 0_u64,
            dg_cg_first: 0_u64,
            dg_data: 0_u64,
            dg_md_comment: 0_u64,
            dg_rec_id_size: 0_u8,
            dg_reserved: [0_u8; 7],
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);
        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let dg_rec_id_size = utils::read(stream, little_endian, &mut pos);
        let dg_reserved = utils::read(stream, little_endian, &mut pos);

        let dg_dg_next = address.remove(0);
        let dg_cg_first = address.remove(0);
        let dg_data = address.remove(0);
        let dg_md_comment = address.remove(0);

        (
            pos,
            Self {
                dg_dg_next,
                dg_cg_first,
                dg_data,
                dg_md_comment,
                dg_rec_id_size,
                dg_reserved,
            },
        )
    }

    fn byte_len(&self) -> usize {
        mem::size_of_val(&self.dg_dg_next)
            + mem::size_of_val(&self.dg_cg_first)
            + mem::size_of_val(&self.dg_data)
            + mem::size_of_val(&self.dg_md_comment)
            + mem::size_of_val(&self.dg_rec_id_size)
    }
}

#[cfg(test)]
mod tests {
    use super::Dgblock;
    use crate::MDF4::block::Block;

    static RAW: [u8; 64] = [
        0x23, 0x23, 0x44, 0x47, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x8D, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x90, 0x86, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA0, 0xA8, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn dg_read_test() {
        let (pos, dg) = Dgblock::read(&RAW, 0, true);

        assert_eq!(64, pos);
        assert_eq!(36336, dg.dg_dg_next);
        assert_eq!(34448, dg.dg_cg_first);
        assert_eq!(43168, dg.dg_data);
        assert_eq!(0, dg.dg_md_comment);
        assert_eq!(0, dg.dg_rec_id_size);
        assert_eq!([0_u8; 7], dg.dg_reserved);
    }
}
