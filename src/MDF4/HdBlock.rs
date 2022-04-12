use crate::MDF4::{Block::Block, BlockHeader::*};
use crate::utils;

use super::DgBlock::Dgblock;
use super::TxBlock;
use super::mdf4::link_extract;


#[derive(Debug, Clone)]
pub struct Hdblock {
    #[allow(dead_code)]
    hd_dg_first: u64,
    #[allow(dead_code)]
    hd_fh_first: u64,
    #[allow(dead_code)]
    hd_ch_first: u64,
    #[allow(dead_code)]
    hd_at_first: u64,
    #[allow(dead_code)]
    hd_ev_first: u64,
    #[allow(dead_code)]
    hd_md_comment: u64,
    #[allow(dead_code)]
    hd_start_time_ns: u64,
    #[allow(dead_code)]
    hd_tz_offset_min: i16,
    #[allow(dead_code)]
    hd_dst_offset_min: i16,
    #[allow(dead_code)]
    hd_time_flags: u8,
    #[allow(dead_code)]
    hd_time_class: u8,
    #[allow(dead_code)]
    hd_flags: u8,
    //hd_reserved: u8,
    #[allow(dead_code)]
    hd_start_angle_rad: f64,
    #[allow(dead_code)]
    hd_start_distance_m: f64,
}

impl Hdblock {
	pub fn first_data_group(&self, stream: &[u8], little_endian: bool) -> Dgblock {
		if self.hd_dg_first == 0 {
			panic!("No data group found!");
		}

		let (_, block) = Dgblock::read(stream, self.hd_dg_first as usize, little_endian);
		block
	}

	pub fn comment(&self, stream: &[u8], little_endian: bool) -> String {
		if self.hd_md_comment == 0 {
			return "".to_string();
		} 

		let (_, tx_block) = TxBlock::Txblock::read(stream, self.hd_md_comment as usize, little_endian);

		tx_block.text()
	}
}


impl Block for Hdblock {
    fn new() -> Self {
        Hdblock {
            hd_dg_first: 0,
            hd_fh_first: 0,
            hd_ch_first: 0,
            hd_at_first: 0,
            hd_ev_first: 0,
            hd_md_comment: 0,
            hd_start_time_ns: 0,
            hd_tz_offset_min: 0,
            hd_dst_offset_min: 0,
            hd_time_flags: 0,
            hd_time_class: 0,
            hd_flags: 0,
            //hd_reserved: 0,
            hd_start_angle_rad: 0.0,
            hd_start_distance_m: 0.0,
        }
    }
    fn default() -> Self {
        Hdblock {
            hd_dg_first: 0,
            hd_fh_first: 0,
            hd_ch_first: 0,
            hd_at_first: 0,
            hd_ev_first: 0,
            hd_md_comment: 0,
            hd_start_time_ns: 0,
            hd_tz_offset_min: 0,
            hd_dst_offset_min: 0,
            hd_time_flags: 0,
            hd_time_class: 0,
            hd_flags: 0,
            //hd_reserved: 0,
            hd_start_angle_rad: 0.0,
            hd_start_distance_m: 0.0,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##HD".as_bytes()) {
            panic!("Error HDBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let hd_dg_first = address.remove(0);
        let hd_fh_first = address.remove(0);
        let hd_ch_first = address.remove(0);
        let hd_at_first = address.remove(0);
        let hd_ev_first = address.remove(0);
        let hd_md_comment = address.remove(0);

        let hd_start_time_ns = utils::read(stream, little_endian, &mut pos);
        let hd_tz_offset_min = utils::read(stream, little_endian, &mut pos);
        let hd_dst_offset_min = utils::read(stream, little_endian, &mut pos);
        let hd_time_flags = utils::read(stream, little_endian, &mut pos);
        let hd_time_class = utils::read(stream, little_endian, &mut pos);
        let hd_flags = utils::read(stream, little_endian, &mut pos);
        let _hd_reserved: u8 = utils::read(stream, little_endian, &mut pos);
        let hd_start_angle_rad = utils::read(stream, little_endian, &mut pos);
        let hd_start_distance_m = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Hdblock {
                hd_dg_first,
                hd_fh_first,
                hd_ch_first,
                hd_at_first,
                hd_ev_first,
                hd_md_comment,
                hd_start_time_ns,
                hd_tz_offset_min,
                hd_dst_offset_min,
                hd_time_flags,
                hd_time_class,
                hd_flags,
                //hd_reserved,
                hd_start_angle_rad,
                hd_start_distance_m,
            },
        )
    }
}