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
    id: Idblock,

    header: Hdblock,

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
                        data_group: dg_no,
                        channel_group: cg_no,
                        channel: cn_no,
                    })
                }
            }
        }

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
                return Ok(i);
            }
        }

        Err("No time series found for the channel group selected")
    }

    fn read_channel(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Vec<Record> {
        let channel_group = &self.channel_groups[channel_grp];
        // let data_length = channel_group.data_length();
        let channels = channel_group
            .first(&self.file, self.little_endian)
            .list(&self.file, self.little_endian);
        let dg = &self.data_groups[datagroup];
        let cn = &channels[channel];

        let data = dg.read_data(&self.file, self.little_endian);
        // &self.file[dg.data_location() as usize..(dg.data_location() as usize + data_length)];

        println!("Record Number: {}", channel_group.record_number());

        let mut data_blocks: Vec<&[u8]> = vec![&[0_u8]; channel_group.record_number()];
        // let mut data_blocks = Vec::new();

        println!("Vec len: {}", data_blocks.len());

        for (i, db) in data_blocks.iter_mut().enumerate() {
            *db = &data[(i * channel_group.record_size())..((i + 1) * channel_group.record_size())];
        }

        let byte_offset = cn.byte_offset();

        let mut records = Vec::new();
        let mut pos = 0;
        for _i in 0..channel_group.record_number() {
            records.push(&data[pos..pos + channel_group.record_size()]);
            pos += channel_group.record_size();
        }

        let mut raw_data = Vec::new();
        let end = byte_offset + cn.data_type_len();

        for rec in records {
            raw_data.push(&rec[byte_offset..end]);
        }

        let mut extracted_data = Vec::new();
        for raw in raw_data {
            extracted_data.push(Record::new(raw, cn.data_type().copy_to_data_type_read()));
        }

        extracted_data
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
                }
            }
        }
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

#[derive(Debug, Clone, PartialEq)]
struct Rdblock {}
