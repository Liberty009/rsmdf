use crate::mdf::{self, MdfChannel};
use crate::mdf3::cg_block::Cgblock;
use crate::mdf3::cn_block::Cnblock;
use crate::record::Record;
use crate::signal;
use std::fs::File;
use std::io::prelude::*;

use super::dg_block::Dgblock;
use super::hd_block::Hdblock;
use super::id_block::Idblock;
use super::mdf3_block::Mdf3Block;
use super::tx_block::Txblock;

// Define constants that are used
const TIME_CHANNEL_TYPE: u16 = 1;

#[derive(Debug, Clone)]
pub struct MDF3 {
    #[allow(dead_code)]
    pub id: Idblock,
    #[allow(dead_code)]
    pub header: Hdblock,
    #[allow(dead_code)]
    pub comment: Txblock,
    pub data_groups: Vec<Dgblock>,
    pub channels: Vec<Cnblock>,
    pub channel_groups: Vec<Cgblock>,
    pub little_endian: bool,
    pub file: Vec<u8>,
}

impl mdf::MDFFile for MDF3 {
    fn channels(&self) -> Vec<MdfChannel> {
        let mut channels = Vec::new();

        let mut dg = Vec::new();
        let mut cg = Vec::new();
        let mut ch = Vec::new();

        let (_id_block, position, little_endian) = Idblock::read(&self.file);
        let (_pos, hd_block) = Hdblock::read(&self.file, position, little_endian);

        let mut next_dg = hd_block.data_group_block;

        while next_dg != 0 {
            let (_pos, dg_block) = Dgblock::read(&self.file, next_dg as usize, little_endian);
            next_dg = dg_block.next;
            let mut next_cg = dg_block.first;

            dg.push(dg_block);

            while next_cg != 0 {
                let (_pos, cg_block) = Cgblock::read(&self.file, next_cg as usize, little_endian);
                next_cg = cg_block.next;
                let mut next_cn = cg_block.first;
                cg.push(cg_block);

                println!("Channel Group: {}", cg_block.comment);

                while next_cn != 0 {
                    let (_pos, cn_block) =
                        Cnblock::read(&self.file, next_cn as usize, little_endian);
                    next_cn = cn_block.next;
                    ch.push(cn_block);

                    let name = cn_block.name(&self.file, little_endian);
                    channels.push(mdf::MdfChannel {
                        name,
                        data_group: (dg.len() - 1) as u64,
                        channel_group: (cg.len() - 1) as u64,
                        channel: (ch.len() - 1) as u64,
                    });
                }
            }
        }

        channels
    }

    fn find_time_channel(
        &self,
        _datagroup: usize,
        channel_grp: usize,
    ) -> Result<usize, &'static str> {
        let channel_group =
            self.channel_groups[channel_grp].channels(&self.file, self.little_endian);
        for (i, channel) in channel_group.iter().enumerate() {
            if channel.channel_type == TIME_CHANNEL_TYPE {
                return Ok(i);
            }
        }

        Err("No time series found for the channel selected")
    }

    fn read_channel(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Vec<Record> {
        let channels: Vec<Cnblock> = self.channel_groups[channel_grp].channels(&self.file, true);
        let data_length = (self.channel_groups[channel_grp].record_number
            * self.channel_groups[channel_grp].record_size as u32)
            as usize;
        let data = &self.file[self.data_groups[datagroup].data_block as usize
            ..(self.data_groups[datagroup].data_block as usize + data_length)];

        println!(
            "Record Number: {}",
            self.channel_groups[channel_grp].record_number
        );

        let mut data_blocks: Vec<&[u8]> =
            vec![&[0]; self.channel_groups[channel_grp].record_number as usize];
        // let mut data_blocks = Vec::with_capacity(self.channel_groups[channel_grp].record_number as usize);
        println!("Vec len: {}", data_blocks.len());

        for (i, db) in data_blocks.iter_mut().enumerate() {
            *db = &data[(i * self.channel_groups[channel_grp].record_size as usize) as usize
                ..((i + 1) * self.channel_groups[channel_grp].record_size as usize) as usize];
        }
        // for i in 0..self.channel_groups[channel_grp].record_number {
        //     data_blocks.push(
        //         &data[(i * self.channel_groups[channel_grp].record_size as u32) as usize
        //             ..((i + 1) * self.channel_groups[channel_grp].record_size as u32) as usize],
        //     );
        // }

        let byte_offset = (self.channels[channel].start_offset / 8) as usize;
        let _bit_offset = self.channels[channel].start_offset % 8;

        let mut records =
            Vec::with_capacity(self.channel_groups[channel_grp].record_number as usize);
        let mut pos = 0_usize;
        for _i in 0..self.channel_groups[channel_grp].record_number {
            records.push(&data[pos..pos + self.channel_groups[channel_grp].record_size as usize]);
            pos += self.channel_groups[channel_grp].record_size as usize;
        }

        let mut raw_data =
            Vec::with_capacity(self.channel_groups[channel_grp].record_number as usize);
        let end = byte_offset + channels[channel].data_type.len();
        for rec in &records {
            raw_data.push(&rec[byte_offset..end])
        }

        let mut extracted_data =
            Vec::with_capacity(self.channel_groups[channel_grp].record_number as usize);
        for raw in raw_data {
            extracted_data.push(Record::new(raw, channels[channel].data_type));
        }

        extracted_data
    }

    fn new(filepath: &str) -> Self {
        let mut file = File::open(filepath).expect("Could not read file");
        let mut stream = Vec::new();
        let _ = file.read_to_end(&mut stream);
        let (id, pos, little_endian) = Idblock::read(&stream);
        let (_pos, header) = Hdblock::read(&stream, pos, little_endian);
        let (_pos, comment) = Txblock::read(&stream, header.file_comment as usize, little_endian);
        let mut mdf = MDF3 {
            id,
            header,
            comment,
            data_groups: Dgblock::read_all(
                &stream,
                little_endian,
                header.data_group_block as usize,
            ),
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
            channel_groups.append(&mut group.read_channel_groups(&self.file, self.little_endian));
        }

        let mut channels = Vec::new();
        for grp in &channel_groups {
            channels.append(&mut grp.channels(&self.file, self.little_endian));
        }

        self.channel_groups = channel_groups;
        self.channels = channels;
    }

    fn list_data_groups(&mut self) {
        let (_id_block, position, little_endian) = Idblock::read(&self.file);
        let (_pos, hd_block) = Hdblock::read(&self.file, position, little_endian);
        //position += pos;

        let dg = Dgblock::read_all(
            &self.file,
            little_endian,
            hd_block.data_group_block as usize,
        );
        self.data_groups = dg;
    }

    fn list_channels(&self) {
        let mut dg = Vec::new();
        let mut cg = Vec::new();
        let mut ch = Vec::new();

        let (_id_block, position, little_endian) = Idblock::read(&self.file);
        let (_pos, hd_block) = Hdblock::read(&self.file, position, little_endian);
        //position += pos;

        let mut next_dg = hd_block.data_group_block;

        while next_dg != 0 {
            let (_pos, dg_block) = Dgblock::read(&self.file, next_dg as usize, little_endian);
            next_dg = dg_block.next;
            let mut next_cg = dg_block.first;

            dg.push(dg_block);

            while next_cg != 0 {
                let (_pos, cg_block) = Cgblock::read(&self.file, next_cg as usize, little_endian);
                next_cg = cg_block.next;
                let mut next_cn = cg_block.first;
                cg.push(cg_block);

                println!("Channel Group: {}", cg_block.comment);

                while next_cn != 0 {
                    let (_pos, cn_block) =
                        Cnblock::read(&self.file, next_cn as usize, little_endian);
                    next_cn = cn_block.next;

                    ch.push(cn_block);
                }
            }
        }

        // (ch, cg, dg);
    }

    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> signal::Signal {
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
    fn resample(&self, _raster: mdf::RasterType, _version: &str, _time_from_zero: bool) -> Self {
        self.clone()
    }
    // fn select(
    //     &self,
    //     _channels: mdf::ChannelsType,
    //     _record_offset: isize,
    //     _raw: bool,
    //     _copy_master: bool,
    //     _ignore_value2text_conversions: bool,
    //     _record_count: isize,
    //     _validate: bool,
    // ) -> Vec<Signal> {
    //     Vec::new()
    // }
}

pub fn print_record(value: Record) {
    match value {
        Record::Uint(number) => print!("{}", number),
        Record::Int(number) => print!("{}", number),
        Record::Float32(number) => print!("{}", number),
        Record::Float64(number) => print!("{}", number),
        // _ => panic!("Help!")
    };
}
