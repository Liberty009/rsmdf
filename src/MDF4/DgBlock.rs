use std::mem;

use super::mdf4::link_extract;
use super::Block::{Block, LinkedBlock};
use super::BlockHeader::*;
use super::CgBlock::Cgblock;
use crate::utils;

#[derive(Debug, Clone)]
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

        let next_block = self;

        all.push(self.clone());
        loop {
            let next_block = next_block.next(stream, little_endian);

            match next_block {
                Some(block) => all.push(block.clone()),
                None => break,
            }
        }

        all
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
        }
    }
    fn default() -> Self {
        Self {
            dg_dg_next: 0_u64,
            dg_cg_first: 0_u64,
            dg_data: 0_u64,
            dg_md_comment: 0_u64,
            dg_rec_id_size: 0_u8,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);
        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let dg_rec_id_size = utils::read(stream, little_endian, &mut pos);
        let _dg_reserved: [u8; 7] = utils::read(stream, little_endian, &mut pos);

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
