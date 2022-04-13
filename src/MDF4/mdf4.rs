use super::cg_block::Cgblock;
use super::cn_block::Cnblock;
use crate::mdf::{self, MDFFile, MdfChannel, RasterType};
use crate::record::Record;
use crate::signal::{self, Signal};
use crate::utils;
use std::fs::File;
use std::io::prelude::*;

use super::block::{Block, LinkedBlock};
use super::dg_block::Dgblock;
use super::hd_block::Hdblock;
use super::id_block::Idblock;
use super::mdf4_enums::ChannelType;

pub fn link_extract(
    stream: &[u8],
    position: usize,
    little_endian: bool,
    no_links: u64,
) -> (usize, Vec<u64>) {
    let mut links = Vec::new();
    let mut pos = position;

    for _i in 0..no_links {
        let address: u64 = utils::read(stream, little_endian, &mut pos);
        links.push(address);
    }

    (pos, links)
}

#[derive(Debug, Clone, PartialEq)]
pub struct MDF4 {
    #[allow(dead_code)]
    id: Idblock,
    #[allow(dead_code)]
    header: Hdblock,
    #[allow(dead_code)]
    comment: String,
    data_groups: Vec<Dgblock>,
    channels: Vec<Cnblock>,
    channel_groups: Vec<Cgblock>,
    little_endian: bool,
    file: Vec<u8>,
}

impl MDFFile for MDF4 {
    fn channels(&self) -> Vec<MdfChannel> {
        let mut mdf_channels = Vec::new();

        let little_endian = true;

        let (position, _id_block) = Idblock::read(&self.file, 0, little_endian);
        let (_pos, hd_block) = Hdblock::read(&self.file, position, little_endian);

        let next_dg = hd_block.first_data_group(&self.file, little_endian);

        let data_groups = next_dg.list(&self.file, little_endian);

        for (dg_no, dg) in data_groups.iter().enumerate() {
            let first_cg = dg.first(&self.file, little_endian);
            let channel_groups = first_cg.list(&self.file, little_endian);

            for (cg_no, cg) in channel_groups.iter().enumerate() {
                let first_cn = cg.first(&self.file, little_endian);
                let channels = first_cn.list(&self.file, little_endian);

                for (cn_no, cn) in channels.iter().enumerate() {
                    let name = cn.clone().comment(&self.file, little_endian);
                    mdf_channels.push(mdf::MdfChannel {
                        name,
                        data_group: dg_no as u64,
                        channel_group: cg_no as u64,
                        channel: cn_no as u64,
                    })
                }
            }
        }

        // while next_dg != 0 {
        //     let (_pos, dg_block) = Dgblock::read(&self.file, next_dg as usize, little_endian);
        //     next_dg = dg_block.dg_dg_next;
        //     let mut next_cg = dg_block.dg_cg_first;

        //     dg.push(dg_block);

        //     while next_cg != 0 {
        //         let (_position, cg_block) =
        //             Cgblock::read(&self.file, next_cg as usize, little_endian);
        //         next_cg = cg_block.cg_cg_next;
        //         let mut next_cn = cg_block.cg_cn_first;
        //         cg.push(cg_block.clone());

        //         println!("Channel Group: {}", cg_block.cg_md_comment);

        //         while next_cn != 0 {
        //             let (_position, cn_block) =
        //                 Cnblock::read(&self.file, next_cn as usize, little_endian);
        //             next_cn = cn_block.cn_cn_next;
        //             ch.push(cn_block.clone());

        //             let name = cn_block.name(&self.file, little_endian);
        //             mdf_channels.push(mdf::MdfChannel {
        //                 name,
        //                 data_group: (dg.len() - 1) as u64,
        //                 channel_group: (cg.len() - 1) as u64,
        //                 channel: (ch.len() - 1) as u64,
        //             });
        //         }
        //     }
        // }

        mdf_channels
    }
    fn find_time_channel(
        &self,
        _datagroup: usize,
        channel_grp: usize,
    ) -> Result<usize, &'static str> {
        let channel_group = self.channel_groups[channel_grp]
            .clone()
            .channels(&self.file, self.little_endian);
        for (i, channel) in channel_group.iter().enumerate() {
            if matches!(channel.channel_type(), ChannelType::Master) {
                //channel.channel_type == TIME_CHANNEL_TYPE {
                return Ok(i);
            }
        }

        Err("No time series found for the channel selected")
    }

    fn read_channel(&self, _datagroup: usize, _channel_grp: usize, _channel: usize) -> Vec<Record> {
        Vec::new()
    }

    #[must_use]
    fn new(filepath: &str) -> Self {
        let mut file = File::open(filepath).expect("Could not read file");
        let mut stream = Vec::new();
        let _ = file.read_to_end(&mut stream);

        let little_endian = true;
        let position = 0;

        let (pos, id) = Idblock::read(&stream, position, little_endian);
        let (_pos, header) = Hdblock::read(&stream, pos, little_endian);
        let comment = header.comment(&stream, little_endian);
        let mut mdf = Self {
            id,
            header: header.clone(),
            comment,
            data_groups: header
                .first_data_group(&stream, little_endian)
                .list(&stream, little_endian),
            channels: Vec::new(),
            channel_groups: Vec::new(),
            little_endian,
            file: stream,
        };

        mdf.read_all();

        mdf
    }

    fn read_all(&mut self) {
        let mut channel_groups = Vec::with_capacity(self.data_groups.len());
        for group in &self.data_groups {
            let group1 = group.clone();
            let mut grp = group1.read_channel_groups(&self.file, self.little_endian);
            channel_groups.append(&mut grp);
        }

        let mut channels = Vec::new();
        for grp in &channel_groups {
            let grp1 = grp.clone();

            channels.append(&mut grp1.channels(&self.file, self.little_endian));
        }

        self.channel_groups = channel_groups;
        self.channels = channels;
    }

    fn list_data_groups(&mut self) {
        let little_endian = true;
        let position = 0;

        let (position, _id_block) = Idblock::read(&self.file, position, little_endian);
        let (_pos, hd_block) = Hdblock::read(&self.file, position, little_endian);
        //position += pos;

        let dg = hd_block
            .first_data_group(&self.file, little_endian)
            .list(&self.file, little_endian);
        self.data_groups = dg;
    }

    fn list_channels(&self) {
        let little_endian = true;

        let (position, _id_block) = Idblock::read(&self.file, 0, little_endian);
        let (_pos, hd_block) = Hdblock::read(&self.file, position, little_endian);

        let next_dg = hd_block.first_data_group(&self.file, little_endian);

        let data_groups = next_dg.list(&self.file, little_endian);

        for dg in data_groups {
            let first_cg = dg.first(&self.file, little_endian);
            let channel_groups = first_cg.list(&self.file, little_endian);

            for cg in channel_groups {
                let first_cn = cg.first(&self.file, little_endian);
                let channels = first_cn.list(&self.file, little_endian);

                println!("Channel Group: {}", cg.comment(&self.file, little_endian));

                for cn in channels {
                    println!("Channel: {}", cn.comment(&self.file, little_endian));
                    //.comment);
                }
            }
        }

        // while next_dg != 0 {
        //     let (_pos, dg_block) = Dgblock::read(&self.file, next_dg as usize, little_endian);
        //     next_dg = dg_block.dg_dg_next; //  .next;
        //     let mut next_cg = dg_block.dg_cg_first; // .first;

        //     dg.push(dg_block);

        //     while next_cg != 0 {
        //         let (_pos, cg_block) = Cgblock::read(&self.file, next_cg as usize, little_endian);
        //         next_cg = cg_block.cg_cg_next; //.next;
        //         let mut next_cn = cg_block.cg_cn_first; //.first;
        //         cg.push(cg_block.clone());

        //         println!("Channel Group: {}", cg_block.cg_md_comment); //.comment);

        //         while next_cn != 0 {
        //             let (_pos, cn_block) =
        //                 Cnblock::read(&self.file, next_cn as usize, little_endian);
        //             next_cn = cn_block.cn_cn_next; //.next;

        //             ch.push(cn_block);
        //         }
        //     }
        // }

        // (ch, cg, dg);
    }

    #[must_use]
    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Signal {
        let time_channel = self.find_time_channel(datagroup, channel_grp);
        let time_channel = match time_channel {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };
        println!("Time Channel: {}", time_channel);
        let time = self.read_channel(datagroup, channel_grp, time_channel);
        let some = self.read_channel(datagroup, channel_grp, channel);

        signal::Signal::new(
            time.iter().map(|x| x.extract()).collect(),
            some.iter().map(|x| x.extract()).collect(),
            "Unit".to_string(),
            "Measurement".to_string(),
            "This is some measurement".to_string(),
            false,
        )
    }

    fn cut(&self, _start: f64, _end: f64, _include_ends: bool, _time_from_zero: bool) {
        // let _delta = if time_from_zero { start } else { 0.0 };
    }

    fn export(&self, _format: &str, _filename: &str) {}
    fn filter(&self, _channels: &str) {}
    #[must_use]
    fn resample(&self, _raster: RasterType, _version: &str, _time_from_zero: bool) -> Self {
        self.clone()
    }
}

// #[derive(Debug, Clone, PartialEq)]
// struct DTBlock {
// 	dt_data: Vec<Record>
// }
// impl Block for DTBlock {
//     fn new() -> Self {
//         Self {}
//     }
//     fn default() -> Self {
//         Self {}
//     }
//     fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
//         (1, Self {})
//     }
// }

#[derive(Debug, Clone, PartialEq)]
struct Rdblock {}

#[derive(Debug, Clone, PartialEq)]
struct Sdblock {}
