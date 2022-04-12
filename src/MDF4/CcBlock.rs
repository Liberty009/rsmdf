use crate::utils;
use crate::MDF4::Block::Block;
use crate::MDF4::BlockHeader::*;

use super::mdf4_enums::CCType;

#[derive(Debug, Clone)]
struct Ccblock {
    #[allow(dead_code)]
    name_addr: u64,
    #[allow(dead_code)]
    unit_addr: u64,
    #[allow(dead_code)]
    comment_addr: u64,
    #[allow(dead_code)]
    inv_conv_addr: u64,
    #[allow(dead_code)]
    cc_ref: Vec<u64>,
    #[allow(dead_code)]
    conversion_type: CCType,
    #[allow(dead_code)]
    precision: u8,
    #[allow(dead_code)]
    flags: u16,
    #[allow(dead_code)]
    ref_param_nr: u16,
    #[allow(dead_code)]
    val_param_nr: u16,
    #[allow(dead_code)]
    min_phy_value: f64,
    #[allow(dead_code)]
    max_phy_value: f64,
    #[allow(dead_code)]
    cc_val: Vec<f64>,
}
impl Block for Ccblock {
    fn new() -> Self {
        Self {
            name_addr: 0,
            unit_addr: 0,
            comment_addr: 0,
            inv_conv_addr: 0,
            cc_ref: Vec::new(),

            conversion_type: CCType::Direct,
            precision: 0,
            flags: 0,
            ref_param_nr: 0,
            val_param_nr: 0,
            min_phy_value: 0.0,
            max_phy_value: 0.0,
            cc_val: Vec::new(),
        }
    }
    fn default() -> Self {
        Self {
            name_addr: 0,
            unit_addr: 0,
            comment_addr: 0,
            inv_conv_addr: 0,
            cc_ref: Vec::new(),

            conversion_type: CCType::Direct,
            precision: 0,
            flags: 0,
            ref_param_nr: 0,
            val_param_nr: 0,
            min_phy_value: 0.0,
            max_phy_value: 0.0,
            cc_val: Vec::new(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (mut pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id[..], "##CC".as_bytes()) {
            panic!("Error: id incorrect");
        }

        let name_addr = utils::read(stream, little_endian, &mut pos);
        let unit_addr = utils::read(stream, little_endian, &mut pos);
        let comment_addr = utils::read(stream, little_endian, &mut pos);
        let inv_conv_addr = utils::read(stream, little_endian, &mut pos);

        let cc_ref_length = (header.link_count - 4) as usize;
        let mut cc_ref = Vec::new();

        for _i in 0..cc_ref_length {
            cc_ref.push(utils::read(stream, little_endian, &mut pos));
        }

        let conversion_type = CCType::new(utils::read(stream, little_endian, &mut pos));
        let precision = utils::read(stream, little_endian, &mut pos);
        let flags = utils::read(stream, little_endian, &mut pos);
        let ref_param_nr = utils::read(stream, little_endian, &mut pos);
        let val_param_nr = utils::read(stream, little_endian, &mut pos);
        let min_phy_value = utils::read(stream, little_endian, &mut pos);
        let max_phy_value = utils::read(stream, little_endian, &mut pos);

        let mut cc_val = Vec::new();
        for _i in 0..val_param_nr {
            cc_val.push(utils::read(stream, little_endian, &mut pos));
        }

        // Check ref count
        assert_eq!(ref_param_nr as usize, cc_ref.len());

        (
            pos,
            Self {
                name_addr,
                unit_addr,
                comment_addr,
                inv_conv_addr,
                conversion_type,
                cc_ref,
                precision,
                flags,
                ref_param_nr,
                val_param_nr,
                min_phy_value,
                max_phy_value,
                cc_val,
            },
        )
    }

    fn byte_len(&self) -> usize {
        1
    }
}

#[test]
fn cc_read_test() {
    let raw: [u8; 96] = [
        0x23, 0x23, 0x43, 0x43, 0x00, 0x00, 0x00, 0x00, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x40,
    ];

    let (pos, cc) = Ccblock::read(&raw, 0, true);

    assert_eq!(96, pos);
}
