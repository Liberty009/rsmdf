use std::mem;

use super::block::Block;
use super::block_header::*;
use crate::utils;
use crate::MDF4::mdf4::link_extract;

use super::mdf4_enums::CCType;

#[derive(Debug, Clone, PartialEq)]
struct Ccblock {
    header: BlockHeader,
    cc_tx_name: u64,
    cc_md_unit: u64,
    cc_md_comment: u64,
    cc_cc_inverse: u64,
    cc_ref: Vec<u64>,
    cc_type: CCType,
    cc_precision: u8,
    cc_flags: u16,
    cc_ref_count: u16,
    cc_val_count: u16,
    cc_phy_range_min: f64,
    cc_phy_range_max: f64,
    cc_val: Vec<f64>,
}
impl Block for Ccblock {
    fn new() -> Self {
        Self {
            header: BlockHeader::create("##CC", 50, 0),
            cc_tx_name: 0_u64,
            cc_md_unit: 0_u64,
            cc_md_comment: 0_u64,
            cc_cc_inverse: 0_u64,
            cc_ref: Vec::new(),
            cc_type: CCType::Direct,
            cc_precision: 0_u8,
            cc_flags: 0_u16,
            cc_ref_count: 0_u16,
            cc_val_count: 0_u16,
            cc_phy_range_min: 0_f64,
            cc_phy_range_max: 0_f64,
            cc_val: Vec::new(),
        }
    }
    fn default() -> Self {
        Self {
            header: BlockHeader::create("##CC", 50, 0),
            cc_tx_name: 0_u64,
            cc_md_unit: 0_u64,
            cc_md_comment: 0_u64,
            cc_cc_inverse: 0_u64,
            cc_ref: Vec::new(),
            cc_type: CCType::Direct,
            cc_precision: 0_u8,
            cc_flags: 0_u16,
            cc_ref_count: 0_u16,
            cc_val_count: 0_u16,
            cc_phy_range_min: 0_f64,
            cc_phy_range_max: 0_f64,
            cc_val: Vec::new(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id[..], "##CC".as_bytes()) {
            panic!("Error: id incorrect");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let cc_tx_name = address.remove(0);
        let cc_md_unit = address.remove(0);
        let cc_md_comment = address.remove(0);
        let cc_cc_inverse = address.remove(0);
        let cc_ref = address;
        let cc_type = CCType::new(utils::read(stream, little_endian, &mut pos));
        let cc_precision = utils::read(stream, little_endian, &mut pos);
        let cc_flags = utils::read(stream, little_endian, &mut pos);
        let cc_ref_count = utils::read(stream, little_endian, &mut pos);
        let cc_val_count = utils::read(stream, little_endian, &mut pos);
        let cc_phy_range_min = utils::read(stream, little_endian, &mut pos);
        let cc_phy_range_max = utils::read(stream, little_endian, &mut pos);

        let mut cc_val = Vec::new();
        for _i in 0..cc_val_count {
            cc_val.push(utils::read(stream, little_endian, &mut pos));
        }

        // Check ref count
        // assert_eq!(cc_ref_count as usize, cc_ref.len());

        (
            pos,
            Self {
                header,
                cc_tx_name,
                cc_md_unit,
                cc_md_comment,
                cc_cc_inverse,
                cc_ref,
                cc_type,
                cc_precision,
                cc_flags,
                cc_ref_count,
                cc_val_count,
                cc_phy_range_min,
                cc_phy_range_max,
                cc_val,
            },
        )
    }

    fn byte_len(&self) -> usize {


        let mut length = self.header.byte_len()
            + mem::size_of_val(&self.cc_tx_name)
            + mem::size_of_val(&self.cc_md_unit)
            + mem::size_of_val(&self.cc_md_comment)
            + mem::size_of_val(&self.cc_cc_inverse)
            + mem::size_of_val(&self.cc_type)
            + mem::size_of_val(&self.cc_precision)
            + mem::size_of_val(&self.cc_flags)
            + mem::size_of_val(&self.cc_ref_count)
            + mem::size_of_val(&self.cc_val_count)
            + mem::size_of_val(&self.cc_phy_range_min)
            + mem::size_of_val(&self.cc_phy_range_max);

        if !&self.cc_ref.is_empty() {
            length += mem::size_of_val(&self.cc_ref[0]) * self.cc_ref_count as usize;
        }

        if !&self.cc_val.is_empty() {
            length += mem::size_of_val(&self.cc_val[0]) * self.cc_val_count as usize;
        }

        length
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use crate::MDF4::{block::Block, cc_block::Ccblock};

    static RAW: [u8; 96] = [
        0x23, 0x23, 0x43, 0x43, 0x00, 0x00, 0x00, 0x00, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x40,
    ];

    #[test]
    fn cc_read_test() {
        let (pos, _cc) = Ccblock::read(&RAW, 0, true);

        assert_eq!(96, pos);
    }

    #[test]
    fn cc_byte_len() {
        let (_pos, cc) = Ccblock::read(&RAW, 0, true);

        println!(
            "cc_val_count: {}, cc_val length: {}",
            cc.cc_val_count,
            cc.cc_val.len()
        );

        assert_eq!(24, cc.header.byte_len());
        assert_eq!(8, mem::size_of_val(&cc.cc_tx_name));
        assert_eq!(8, mem::size_of_val(&cc.cc_md_unit));
        assert_eq!(8, mem::size_of_val(&cc.cc_md_comment));
        assert_eq!(8, mem::size_of_val(&cc.cc_cc_inverse));
        if !cc.cc_ref.is_empty() {
            assert_eq!(
                cc.cc_ref_count as usize * 8,
                mem::size_of_val(&cc.cc_ref[0]) * cc.cc_ref_count as usize
            );
        }
        assert_eq!(1, mem::size_of_val(&cc.cc_type));
        assert_eq!(1, mem::size_of_val(&cc.cc_precision));
        assert_eq!(2, mem::size_of_val(&cc.cc_flags));
        assert_eq!(2, mem::size_of_val(&cc.cc_ref_count));
        assert_eq!(2, mem::size_of_val(&cc.cc_val_count));
        assert_eq!(8, mem::size_of_val(&cc.cc_phy_range_min));
        assert_eq!(8, mem::size_of_val(&cc.cc_phy_range_max));

        if !cc.cc_val.is_empty() {
            assert_eq!(
                cc.cc_val_count as usize * 8,
                mem::size_of_val(&cc.cc_val[0]) * cc.cc_val_count as usize
            );
        }

        assert_eq!(96, cc.byte_len());
    }
}
