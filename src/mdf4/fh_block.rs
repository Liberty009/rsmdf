use std::mem;

use super::block::Block;
use super::block_header::*;
use super::mdf4_file::link_extract;
use crate::utils;

#[derive(Debug, Clone, PartialEq)]
struct Fhblock {
    header: BlockHeader,

    fh_fh_next: u64,

    fh_md_comment: u64,

    fh_time_ns: u64,

    fh_tz_offset_min: i16,

    fh_dst_offset_min: i16,

    fh_time_flags: u8,

    fh_reserved: [u8; 3],
}
impl Block for Fhblock {
    fn new() -> Self {
        Self {
            header: BlockHeader::create("##FH", 50, 0),
            fh_fh_next: 0_u64,
            fh_md_comment: 0_u64,
            fh_time_ns: 0_u64,
            fh_tz_offset_min: 0_i16,
            fh_dst_offset_min: 0_i16,
            fh_time_flags: 0_u8,
            fh_reserved: [0_u8; 3],
        }
    }
    fn default() -> Self {
        Self {
            header: BlockHeader::create("##FH", 50, 0),
            fh_fh_next: 0_u64,
            fh_md_comment: 0_u64,
            fh_time_ns: 0_u64,
            fh_tz_offset_min: 0_i16,
            fh_dst_offset_min: 0_i16,
            fh_time_flags: 0_u8,
            fh_reserved: [0_u8; 3],
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##FH".as_bytes()) {
            panic!("Error FHBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let fh_fh_next = address.remove(0);
        let fh_md_comment = address.remove(0);

        let fh_time_ns = utils::read(stream, little_endian, &mut pos);
        let fh_tz_offset_min = utils::read(stream, little_endian, &mut pos);
        let fh_dst_offset_min = utils::read(stream, little_endian, &mut pos);
        let fh_time_flags = utils::read(stream, little_endian, &mut pos);
        let fh_reserved = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                header,
                fh_fh_next,
                fh_md_comment,
                fh_time_ns,
                fh_tz_offset_min,
                fh_dst_offset_min,
                fh_time_flags,
                fh_reserved,
            },
        )
    }

    fn byte_len(&self) -> usize {
        self.header.byte_len()
            + mem::size_of_val(&self.fh_fh_next)
            + mem::size_of_val(&self.fh_md_comment)
            + mem::size_of_val(&self.fh_time_ns)
            + mem::size_of_val(&self.fh_tz_offset_min)
            + mem::size_of_val(&self.fh_dst_offset_min)
            + mem::size_of_val(&self.fh_time_flags)
            + mem::size_of_val(&self.fh_reserved)
    }
}

#[cfg(test)]
mod tests {
    use crate::mdf4::{block::Block, fh_block::Fhblock};

    static RAW: [u8; 56] = [
        0x23, 0x23, 0x46, 0x48, 0x00, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0xC8, 0x11, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4E, 0x10, 0xDF, 0x75,
        0x78, 0x69, 0x15, 0x3C, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn read() {
        let (pos, fh) = Fhblock::read(&RAW, 0, true);

        assert_eq!(pos, RAW.len());
        assert_eq!(1165408, fh.fh_fh_next);
        assert_eq!(224, fh.fh_md_comment);
        //assert_eq!(42896795000000000, fh.fh_time_ns);
        assert_eq!(60, fh.fh_tz_offset_min);
        assert_eq!(0, fh.fh_dst_offset_min);
        assert_eq!(2, fh.fh_time_flags);
    }

    #[test]
    fn byte_len() {
        let (pos, fh) = Fhblock::read(&RAW, 0, true);

        assert_eq!(pos, fh.byte_len());
    }
}
