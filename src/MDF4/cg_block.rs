use std::mem;

use super::block::Block;
use super::block_header::*;
use crate::utils;

use super::block::LinkedBlock;
use super::cn_block::Cnblock;
use super::mdf4::link_extract;
use super::tx_block;

#[derive(Debug, Clone, PartialEq)]
pub struct Cgblock {
    header: BlockHeader,
    #[allow(dead_code)]
    cg_cg_next: u64, //- int : next channel group address
    #[allow(dead_code)]
    cg_cn_first: u64, //- int : address of first channel of this channel group
    #[allow(dead_code)]
    cg_tx_acq_name: u64, //- int : address of TextBLock that contains the channel
    #[allow(dead_code)]
    cg_si_acq_source: u64, //- int : address of SourceInformation that contains the
    #[allow(dead_code)]
    cg_sr_first: u64, // - int : address of first SRBLOCK; this is
    #[allow(dead_code)]
    cg_md_comment: u64, //- int : address of TXBLOCK/MDBLOCK that contains the
    #[allow(dead_code)]
    cg_record_id: u64, //- int : record ID for the channel group
    #[allow(dead_code)]
    cg_cycle_count: u64, //- int : number of cycles for this channel group
    #[allow(dead_code)]
    cg_flags: u16, //- int : channel group flags
    #[allow(dead_code)]
    cg_path_separator: u16,
    cg_reserved: [u8; 4],
    #[allow(dead_code)]
    cg_data_bytes: u32,
    #[allow(dead_code)]
    cg_inval_bytes: u32, // - int : number of bytes used for invalidation
                         // bits by this channel group

                         //Other attributes
                         //acq_name: u64,   // - str : acquisition name
                         //acq_source: u64, //- SourceInformation : acquisition source information
                         //address: u64,    //- int : channel group address
                         //comment: u64,    //- str : channel group comment
}

impl LinkedBlock for Cgblock {
    fn next(&self, stream: &[u8], little_endian: bool) -> Option<Self> {
        if self.cg_cg_next == 0 {
            None
        } else {
            let (_, block) = Self::read(stream, self.cg_cg_next as usize, little_endian);
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
    }
}

impl Cgblock {
    pub fn first(&self, stream: &[u8], little_endian: bool) -> Cnblock {
        let (_, block) = Cnblock::read(stream, self.cg_cn_first as usize, little_endian);
        block
    }

    pub fn channels(self, stream: &[u8], little_endian: bool) -> Vec<Cnblock> {
        //let mut ch = Vec::new();
        let first = self.first(stream, little_endian);
        // ch.push(first.clone());

        // let next = first;
        // loop {
        //     let next = next.next(stream, little_endian);

        //     match next {
        //         Some(cn) => ch.push(cn.clone()),
        //         None => break,
        //     }
        // }
        // ch

        first.list(stream, little_endian)
    }

    pub fn comment(&self, stream: &[u8], little_endian: bool) -> String {
        if self.cg_md_comment == 0 {
            return "".to_string();
        }

        let (_, tx_block) =
            tx_block::Txblock::read(stream, self.cg_md_comment as usize, little_endian);

        tx_block.text()
    }
}

impl Block for Cgblock {
    fn new() -> Self {
        Cgblock {
            header: BlockHeader::create("##CG", 50, 0),
            cg_cg_next: 0,
            cg_cn_first: 0,
            cg_tx_acq_name: 0,
            cg_si_acq_source: 0,
            cg_sr_first: 0,
            cg_md_comment: 0,
            cg_record_id: 0,
            cg_cycle_count: 0,
            cg_flags: 0,
            cg_path_separator: 0,
            cg_reserved: [0_u8; 4],
            cg_data_bytes: 0,
            cg_inval_bytes: 0,
        }
    }
    fn default() -> Self {
        Cgblock {
            header: BlockHeader::create("##CG", 50, 0),
            cg_cg_next: 0,
            cg_cn_first: 0,
            cg_tx_acq_name: 0,
            cg_si_acq_source: 0,
            cg_sr_first: 0,
            cg_md_comment: 0,
            cg_record_id: 0,
            cg_cycle_count: 0,
            cg_flags: 0,
            cg_path_separator: 0,
            cg_reserved: [0_u8; 4],
            cg_data_bytes: 0,
            cg_inval_bytes: 0,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##CG".as_bytes()) {
            panic!("Error: Channel group wrong id");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let next_cg_addr = address.remove(0);
        let first_ch_addr = address.remove(0);
        let acq_name_addr = address.remove(0);
        let acq_source_addr = address.remove(0);
        let first_sample_reduction_addr = address.remove(0);
        let comment_addr = address.remove(0);

        let record_id = utils::read(stream, little_endian, &mut pos);
        let cycles_nr = utils::read(stream, little_endian, &mut pos);
        let flags = utils::read(stream, little_endian, &mut pos);
        let path_separator = utils::read(stream, little_endian, &mut pos);
        let cg_reserved = utils::read(stream, little_endian, &mut pos);
        let samples_byte_nr = utils::read(stream, little_endian, &mut pos);
        let invalidation_bytes_nr = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Cgblock {
                header,
                cg_cg_next: next_cg_addr,
                cg_cn_first: first_ch_addr,
                cg_tx_acq_name: acq_name_addr,
                cg_si_acq_source: acq_source_addr,
                cg_sr_first: first_sample_reduction_addr,
                cg_md_comment: comment_addr,
                cg_record_id: record_id,
                cg_cycle_count: cycles_nr,
                cg_flags: flags,
                cg_path_separator: path_separator,
                cg_reserved,
                cg_data_bytes: samples_byte_nr,
                cg_inval_bytes: invalidation_bytes_nr,
                // acq_name,
                // comment,
            },
        )
    }

    fn byte_len(&self) -> usize {
        self.header.byte_len() + 
        mem::size_of_val(&self.cg_cg_next)
            + mem::size_of_val(&self.cg_cn_first)
            + mem::size_of_val(&self.cg_tx_acq_name)
            + mem::size_of_val(&self.cg_si_acq_source)
            + mem::size_of_val(&self.cg_sr_first)
            + mem::size_of_val(&self.cg_md_comment)
            + mem::size_of_val(&self.cg_record_id)
            + mem::size_of_val(&self.cg_cycle_count)
            + mem::size_of_val(&self.cg_flags)
            + mem::size_of_val(&self.cg_path_separator)
            + mem::size_of_val(&self.cg_data_bytes)
            + mem::size_of_val(&self.cg_inval_bytes)
            + mem::size_of_val(&self.cg_reserved)
    }
}

#[cfg(test)]
mod tests {
    use crate::MDF4::{block::Block, cg_block::Cgblock};

    static RAW: [u8; 104] = [
        0x23, 0x23, 0x43, 0x47, 0x00, 0x00, 0x00, 0x00, 0x68, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xA0, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD0, 0x3F, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xF0, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0xF8, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0xD8, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn read() {
        let (pos, cg) = Cgblock::read(&RAW, 0, true);

        assert_eq!(pos, 104);
        assert_eq!(cg.cg_cg_next, 0);
        assert_eq!(cg.cg_cn_first, 25760);
        assert_eq!(cg.cg_tx_acq_name, 16336);
        assert_eq!(cg.cg_si_acq_source, 26352);
        assert_eq!(cg.cg_sr_first, 0);
        assert_eq!(cg.cg_md_comment, 16376);

        assert_eq!(cg.cg_record_id, 0);
        assert_eq!(cg.cg_cycle_count, 1240);
        assert_eq!(cg.cg_flags, 0);
        // assert_eq!(cg.cg_path_separator, 10);
        assert_eq!(cg.cg_data_bytes, 10);
    }

    #[test]
    fn byte_len() {
        let (pos, cg) = Cgblock::read(&RAW, 0, true);

        assert_eq!(pos, cg.byte_len());

    }
}
