use crate::mdf::{self, MdfChannel};
use crate::record::{DataType, DataTypeRead, Record};
use crate::{signal, utils};
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;

// Define constants that are used
const TIME_CHANNEL_TYPE: u16 = 1;

const UNSIGNED_INT_DEFAULT: u16 = 0;
const SIGNED_INT_DEFAULT: u16 = 1;
const FLOAT32_DEFAULT: u16 = 2;
const FLOAT64_DEFAULT: u16 = 3;
const FFLOAT_DEFAULT: u16 = 4;
const GFLOAT_DEFAULT: u16 = 5;
const DFLOAT_DEFAULT: u16 = 6;
const STRING_NULL_TERM: u16 = 7;
const BYTE_ARRAY: u16 = 8;
const UNSIGNED_INT_BIGENDIAN: u16 = 9;
const SIGNED_INT_BIGENDIAN: u16 = 10;
const FLOAT32_BIGENDIAN: u16 = 11;
const FLOAT64_BIGENDIAN: u16 = 12;
const UNSIGNED_INT_LITTLEENDIAN: u16 = 13;
const SIGNED_INT_LITTLEENDIAN: u16 = 14;
const FLOAT32_INT_LITTLEENDIAN: u16 = 15;
const FLOAT64_INT_LITTLEENDIAN: u16 = 16;

#[derive(Debug, Clone)]
pub struct MDF3 {
    #[allow(dead_code)]
    pub id: IDBLOCK,
    #[allow(dead_code)]
    pub header: HDBLOCK,
    #[allow(dead_code)]
    pub comment: TXBLOCK,
    pub data_groups: Vec<DGBLOCK>,
    pub channels: Vec<CNBLOCK>,
    pub channel_groups: Vec<CGBLOCK>,
    pub little_endian: bool,
    pub file: Vec<u8>,
}

impl mdf::MDFFile for MDF3 {
    fn channels(&self) -> Vec<MdfChannel> {
        let mut channels = Vec::new();

        let mut dg = Vec::new();
        let mut cg = Vec::new();
        let mut ch = Vec::new();

        let (_id_block, position, little_endian) = IDBLOCK::read(&self.file);
        let (hd_block, _pos) = HDBLOCK::read(&self.file, position, little_endian);

        let mut next_dg = hd_block.data_group_block;

        while next_dg != 0 {
            let dg_block = DGBLOCK::read(&self.file, little_endian, &mut (next_dg as usize));
            next_dg = dg_block.next;
            let mut next_cg = dg_block.first;

            dg.push(dg_block);

            while next_cg != 0 {
                let (cg_block, _position) =
                    CGBLOCK::read(&self.file, little_endian, next_cg as usize);
                next_cg = cg_block.next;
                let mut next_cn = cg_block.first;
                cg.push(cg_block);

                println!("Channel Group: {}", cg_block.comment);

                while next_cn != 0 {
                    let (cn_block, _position) =
                        CNBLOCK::read(&self.file, little_endian, next_cn as usize);
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
        let channels: Vec<CNBLOCK> = self.channel_groups[channel_grp].channels(&self.file, true);
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
        let (id, pos, little_endian) = IDBLOCK::read(&stream);
        let (header, _pos) = HDBLOCK::read(&stream, pos, little_endian);
        let (comment, _pos) = TXBLOCK::read(&stream, header.file_comment as usize, little_endian);
        let mut mdf = MDF3 {
            id,
            header,
            comment,
            data_groups: DGBLOCK::read_all(
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

    fn list(&mut self) {
        let (_id_block, position, little_endian) = IDBLOCK::read(&self.file);
        let (hd_block, _pos) = HDBLOCK::read(&self.file, position, little_endian);
        //position += pos;

        let dg = DGBLOCK::read_all(
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

        let (_id_block, position, little_endian) = IDBLOCK::read(&self.file);
        let (hd_block, _pos) = HDBLOCK::read(&self.file, position, little_endian);
        //position += pos;

        let mut next_dg = hd_block.data_group_block;

        while next_dg != 0 {
            let dg_block = DGBLOCK::read(&self.file, little_endian, &mut (next_dg as usize));
            next_dg = dg_block.next;
            let mut next_cg = dg_block.first;

            dg.push(dg_block);

            while next_cg != 0 {
                let (cg_block, _position) =
                    CGBLOCK::read(&self.file, little_endian, next_cg as usize);
                next_cg = cg_block.next;
                let mut next_cn = cg_block.first;
                cg.push(cg_block);

                println!("Channel Group: {}", cg_block.comment);

                while next_cn != 0 {
                    let (cn_block, _position) =
                        CNBLOCK::read(&self.file, little_endian, next_cn as usize);
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

#[derive(Debug, Clone)]
pub struct IDBLOCK {
    pub file_id: [u8; 8],
    pub format_id: [u8; 8],
    pub program_id: [u8; 8],
    pub default_byte_order: u16,
    pub default_float_format: u16,
    pub version_number: u16,
    pub code_page_number: u16,
    pub reserved1: [u8; 2],
    pub reserved2: [u8; 30],
}

impl IDBLOCK {
    pub fn read(stream: &[u8]) -> (IDBLOCK, usize, bool) {
        let mut position = 0;
        let file_id: [u8; 8] = stream[position..position + 8].try_into().expect("msg");

        if !utils::eq(
            &file_id[..],
            &[0x4D, 0x44, 0x46, 0x20, 0x20, 0x20, 0x20, 0x20],
        ) {
            panic!("Error: Incorrect file type");
        }

        position += file_id.len();

        let format_id: [u8; 8] = stream[position..position + 8].try_into().expect("msg");
        position += format_id.len();

        let program_id: [u8; 8] = stream[position..position + 8].try_into().expect("msg");
        position += program_id.len();

        let default_byte_order = utils::read(stream, true, &mut position);

        let little_endian = default_byte_order == 0;

        let default_float_format = utils::read(stream, little_endian, &mut position);

        let version_number = utils::read(stream, little_endian, &mut position);

        let code_page_number = utils::read(stream, little_endian, &mut position);

        let reserved1: [u8; 2] = [stream[position], stream[position + 1]];
        position += reserved1.len();
        let reserved2: [u8; 30] = stream[position..position + 30].try_into().expect("msg");
        position += reserved2.len();

        (
            IDBLOCK {
                file_id,
                format_id,
                program_id,
                default_byte_order,
                default_float_format,
                version_number,
                code_page_number,
                reserved1,
                reserved2,
            },
            position,
            little_endian,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HDBLOCK {
    pub position: usize,
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub data_group_block: u32,
    pub file_comment: u32,
    pub program_block: u32,
    pub data_group_number: u16,
    pub date: [u8; 10],
    pub time: [u8; 8],
    pub author: [u8; 32],
    pub department: [u8; 32],
    pub project: [u8; 32],
    pub subject: [u8; 32],
    pub timestamp: u64,
    pub utc_time_offset: i16,
    pub time_quality: u16,
    pub timer_id: [u8; 32],
}

impl HDBLOCK {
    pub fn read(stream: &[u8], position: usize, little_endian: bool) -> (HDBLOCK, usize) {
        let mut pos = position;
        let block_type: [u8; 2] = stream[position..position + 2].try_into().expect("");

        if !utils::eq(&block_type, &[b'H', b'D']) {
            panic!("Incorrect type for HDBLOCK");
        }

        pos += block_type.len();
        let block_size = utils::read(stream, little_endian, &mut pos);
        let data_group_block = utils::read(stream, little_endian, &mut pos);
        let file_comment = utils::read(stream, little_endian, &mut pos);
        let program_block = utils::read(stream, little_endian, &mut pos);
        let data_group_number = utils::read(stream, little_endian, &mut pos);
        let date: [u8; 10] = stream[pos..pos + 10].try_into().expect("msg");
        pos += date.len();
        let time: [u8; 8] = stream[pos..pos + 8].try_into().expect("msg");
        pos += time.len();
        let author: [u8; 32] = stream[pos..pos + 32].try_into().expect("msg");
        pos += author.len();
        let department: [u8; 32] = stream[pos..pos + 32].try_into().expect("msg");
        pos += department.len();
        let project: [u8; 32] = stream[pos..pos + 32].try_into().expect("msg");
        pos += project.len();
        let subject: [u8; 32] = stream[pos..pos + 32].try_into().expect("msg");
        pos += subject.len();
        let timestamp = utils::read(stream, little_endian, &mut pos);
        let utc_time_offset = utils::read(stream, little_endian, &mut pos);
        let time_quality = utils::read(stream, little_endian, &mut pos);
        let timer_id: [u8; 32] = stream[pos..pos + 32].try_into().expect("msg");
        pos += timer_id.len();

        (
            HDBLOCK {
                position,
                block_type,
                block_size,
                data_group_block,
                file_comment,
                program_block,
                data_group_number,
                date,
                time,
                author,
                department,
                project,
                subject,
                timestamp,
                utc_time_offset,
                time_quality,
                timer_id,
            },
            pos,
        )
    }
}

#[derive(Debug, Clone)]
pub struct TXBLOCK {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub text: Vec<u8>,
}

impl TXBLOCK {
    pub fn read(stream: &[u8], position: usize, little_endian: bool) -> (TXBLOCK, usize) {
        let mut pos = position;

        let block_type: [u8; 2] = stream[pos..pos + 2].try_into().expect("");
        if !utils::eq(&block_type, &[b'T', b'X']) {
            panic!(
                "TXBLOCK type incorrect. Found : {}, {}",
                block_type[0], block_type[1]
            );
        }

        pos += block_type.len();
        let block_size = utils::read(stream, little_endian, &mut pos);

        let mut text: Vec<u8> = stream[pos..pos + block_size as usize - 5]
            .try_into()
            .expect("msg");

        // make sure that the text is utf8
        for c in &mut text {
            if 128 < *c {
                *c = 32;
            }
        }
        pos += text.len();

        (
            TXBLOCK {
                block_type,
                block_size,
                text,
            },
            pos,
        )
    }

    pub fn name(self) -> String {
        //let mut name = "".to_string();

        //let (tx, _pos) = Self::read(stream, little_endian);

        utils::extract_name(&self.text)
    }
}

#[derive(Debug, Clone)]
pub struct PRBLOCK {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub program_data: Vec<u8>,
}

impl PRBLOCK {
    pub fn read(stream: &[u8], position: usize, little_endian: bool) -> (PRBLOCK, usize) {
        let mut pos = position;
        let block_type: [u8; 2] = stream[pos..pos + 2].try_into().expect("");
        if !utils::eq(&block_type, &[b'P', b'R']) {
            panic!("PR Block not found");
        }

        pos += block_type.len();

        let block_size = utils::read(stream, little_endian, &mut pos);
        pos += 2;

        //let mut program_data = vec![0; block_size as usize];
        let mut program_data: Vec<u8> = stream[pos..block_size as usize - pos]
            .try_into()
            .expect("msg");

        pos += program_data.len();

        // make sure that the text is utf8
        for c in &mut program_data {
            if 128 <= *c {
                *c = 32;
            }
        }

        (
            PRBLOCK {
                block_type,
                block_size,
                program_data,
            },
            pos,
        )
    }
}

#[derive(Debug, Clone)]
pub struct TRBLOCK {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub trigger_comment: u32,
    pub trigger_events_number: u16,
    pub events: Vec<Event>,
}

impl TRBLOCK {
    pub fn read(stream: &[u8], little_endian: bool, position: usize) -> (TRBLOCK, usize) {
        let mut pos = position;

        let block_type: [u8; 2] = stream[pos..pos + 2].try_into().expect("msg");
        if !utils::eq(&block_type, &[b'T', b'R']) {
            panic!(
                "TRBLOCK not found. Found: {}, {}",
                block_type[0], block_type[1]
            );
        }

        pos += block_type.len();

        let block_size = utils::read(&stream[pos..], little_endian, &mut pos);
        let trigger_comment = utils::read(&stream[pos..], little_endian, &mut pos);
        let trigger_events_number = utils::read(stream, little_endian, &mut pos);
        let (events, pos) = TRBLOCK::read_events(stream, pos, little_endian, trigger_events_number);

        (
            TRBLOCK {
                block_type,
                block_size,
                trigger_comment,
                trigger_events_number,
                events,
            },
            pos,
        )
    }

    fn read_events(
        stream: &[u8],
        position: usize,
        little_endian: bool,
        no_events: u16,
    ) -> (Vec<Event>, usize) {
        let mut events = Vec::with_capacity(no_events as usize + 1);
        let mut pos1 = position;
        for _i in 0..no_events {
            let (event, pos) = Event::read(stream, pos1, little_endian);
            events.push(event);
            pos1 += pos;
        }

        (events, position)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub trigger_time: f64,
    pub pre_trigger_time: f64,
    pub post_trigger_time: f64,
}

impl Event {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (Event, usize) {
        let mut pos = position;
        let trigger_time = utils::read(stream, little_endian, &mut pos);
        let pre_trigger_time = utils::read(stream, little_endian, &mut pos);
        let post_trigger_time = utils::read(stream, little_endian, &mut pos);
        (
            Event {
                trigger_time,
                pre_trigger_time,
                post_trigger_time,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SRBLOCK {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub next: u32,
    pub data_block: u32,
    pub samples_reduced_number: u32,
    pub time_interval_length: f64,
}

impl SRBLOCK {
    pub fn read(stream: &[u8], little_endian: bool) -> (SRBLOCK, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream.try_into().expect("msg");
        position += block_type.len();
        let block_size = utils::read(stream, little_endian, &mut position);
        let next = utils::read(stream, little_endian, &mut position);
        let data_block = utils::read(stream, little_endian, &mut position);
        let samples_reduced_number = utils::read(stream, little_endian, &mut position);
        let time_interval_length = utils::read(stream, little_endian, &mut position);

        (
            SRBLOCK {
                block_type,
                block_size,
                next,
                data_block,
                samples_reduced_number,
                time_interval_length,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DGBLOCK {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub next: u32,
    pub first: u32,
    pub trigger_block: u32,
    pub data_block: u32,
    pub group_number: u16,
    pub id_number: u16,
    pub reserved: u32,
}

impl DGBLOCK {
    // Read the data stream in to a DGBLOCK type, position reached
    pub fn read(stream: &[u8], little_endian: bool, position: &mut usize) -> Self {
        let pos = position;

        // Read block type to confirm
        let block_type: [u8; 2] = stream[*pos..*pos + 2].try_into().expect("msg");
        if !utils::eq(&block_type, &[b'D', b'G']) {
            panic!(
                "DGBLOCK not found. Found: {}, {}",
                block_type[0], block_type[1]
            );
        }

        *pos += block_type.len();

        let block_size = utils::read(stream, little_endian, pos);
        let next = utils::read(stream, little_endian, pos);
        let first = utils::read(stream, little_endian, pos);
        let trigger_block = utils::read(stream, little_endian, pos);
        let data_block = utils::read(stream, little_endian, pos);
        let group_number = utils::read(stream, little_endian, pos);
        let id_number = utils::read(stream, little_endian, pos);
        let reserved = utils::read(stream, little_endian, pos);

        DGBLOCK {
            block_type,
            block_size,
            next,
            first,
            trigger_block,
            data_block,
            group_number,
            id_number,
            reserved,
        }
    }

    pub fn read_all(stream: &[u8], little_endian: bool, position: usize) -> Vec<Self> {
        let mut all = Vec::new();
        let mut next_dg = position;

        while next_dg != 0 {
            let dg_block = DGBLOCK::read(stream, little_endian, &mut next_dg);
            next_dg = dg_block.next as usize;
            all.push(dg_block);
        }

        all
    }

    pub fn read_channel_groups(self, stream: &[u8], little_endian: bool) -> Vec<CGBLOCK> {
        let mut channel_grps = Vec::new();
        let mut next = self.first as usize;
        while next != 0 {
            let (cg_block, _pos) = CGBLOCK::read(stream, little_endian, next);
            next = cg_block.next as usize;
            channel_grps.push(cg_block);
        }
        channel_grps
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CGBLOCK {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub next: u32,
    pub first: u32,
    pub comment: u32,
    pub record_id: u16,
    pub channel_number: u16,
    pub record_size: u16,
    pub record_number: u32,
    pub first_sample_reduction_block: u32,
}

impl CGBLOCK {
    pub fn read(stream: &[u8], little_endian: bool, position: usize) -> (CGBLOCK, usize) {
        let mut pos = position;
        let block_type: [u8; 2] = stream[pos..pos + 2].try_into().expect("msg");

        if !utils::eq(&block_type, &[b'C', b'G']) {
            panic!(
                "CGBLOCK not found. Found: {}, {}",
                block_type[0] as char, block_type[1] as char
            );
        }

        pos += block_type.len();

        let block_size = utils::read(stream, little_endian, &mut pos);
        let next = utils::read(stream, little_endian, &mut pos);
        let first = utils::read(stream, little_endian, &mut pos);
        let comment = utils::read(stream, little_endian, &mut pos);
        let record_id = utils::read(stream, little_endian, &mut pos);
        let channel_number = utils::read(stream, little_endian, &mut pos);
        let record_size = utils::read(stream, little_endian, &mut pos);
        let record_number = utils::read(stream, little_endian, &mut pos);
        let first_sample_reduction_block = utils::read(stream, little_endian, &mut pos);

        (
            CGBLOCK {
                block_type,
                block_size,
                next,
                first,
                comment,
                record_id,
                channel_number,
                record_size,
                record_number,
                first_sample_reduction_block,
            },
            pos,
        )
    }
    pub fn channels(self, stream: &[u8], little_endian: bool) -> Vec<CNBLOCK> {
        //let (group, _) = Self::read(stream, little_endian, position);
        let mut ch = Vec::new();
        let mut next_cn = self.first as usize;
        while next_cn != 0 {
            let (cn_block, _position) = CNBLOCK::read(stream, little_endian, next_cn);
            next_cn = cn_block.next as usize;

            ch.push(cn_block);
        }

        ch
    }
}

// pub struct RecordedData<T: utils::FromBytes> {
//     data: Vec<T>,
// }

// impl<T: utils::FromBytes> RecordedData<T> {
//     fn new(stream: &[&[u8]], dtype: DataTypeRead) -> Self {
//         let mut result = Vec::new();

// 		for value in stream {
//             result.push(utils::read(value, dtype.little_endian, &mut 0))
//         }

//         RecordedData{ data: result};
//     }

// }

// struct Number<T: utils::FromBytes> {
//     data: Vec<T>,
// }

// impl<T: utils::FromBytes> Number<T> {
//     fn new(stream: &[&[u8]], dtype: DataTypeRead) -> Self {
//         let mut converted: Vec<T> = Vec::new();
//         for value in stream {
//             converted.push(utils::read(value, dtype.little_endian, &mut 0));
//         }

//         Number { data: converted };
//     }
// }
// struct StringNull {
//     StringNull: String,
// }
// impl StringNull {
//     fn new() -> Self {
//         StringNull {
//             StringNull: "".to_string(),
//         };
//     }
// }
// struct Array {}

// impl Array {
//     fn new() -> Self {
//         Array {}
//     }
// }

#[derive(Debug, Clone, Copy)]
pub struct CNBLOCK {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub next: u32,
    pub conversion_formula: u32,
    pub source_ext: u32,
    pub dependency: u32,
    pub comment: u32,
    pub channel_type: u16,
    pub short_name: [u8; 32],
    pub desc: [u8; 128],
    pub start_offset: u16,
    pub bit_number: u16,
    pub data_type: DataTypeRead,
    pub value_range_valid: u16,
    pub signal_min: f64,
    pub signal_max: f64,
    pub sample_rate: f64,
    pub long_name: u32,
    pub display_name: u32,
    pub addition_byte_offset: u16,
}

impl CNBLOCK {
    pub fn read(stream: &[u8], little_endian: bool, position: usize) -> (CNBLOCK, usize) {
        let mut pos = position;
        let block_type: [u8; 2] = stream[pos..pos + 2].try_into().expect("msg");
        pos += block_type.len();
        if !utils::eq(&block_type, &[b'C', b'N']) {
            panic!("CNBLOCK not found.");
        }

        let block_size = utils::read(stream, little_endian, &mut pos);
        let next = utils::read(stream, little_endian, &mut pos);
        let conversion_formula = utils::read(stream, little_endian, &mut pos);
        let source_ext = utils::read(stream, little_endian, &mut pos);
        let dependency = utils::read(stream, little_endian, &mut pos);
        let comment = utils::read(stream, little_endian, &mut pos);
        let channel_type = utils::read(stream, little_endian, &mut pos);

        let short_name: [u8; 32] = stream[pos..pos + 32].try_into().expect("msg");
        pos += short_name.len();

        let desc: [u8; 128] = stream[pos..pos + 128].try_into().expect("msg");
        pos += desc.len();

        let start_offset = utils::read(stream, little_endian, &mut pos);
        let bit_number = utils::read(stream, little_endian, &mut pos);

        let datatype: u16 = utils::read(stream, little_endian, &mut pos);
        let data_type = match datatype {
            UNSIGNED_INT_DEFAULT => DataTypeRead {
                data_type: DataType::UnsignedInt,
                little_endian,
            },
            SIGNED_INT_DEFAULT => DataTypeRead {
                data_type: DataType::SignedInt,
                little_endian,
            },
            FLOAT32_DEFAULT => DataTypeRead {
                data_type: DataType::Float32,
                little_endian,
            },
            FLOAT64_DEFAULT => DataTypeRead {
                data_type: DataType::Float64,
                little_endian,
            },
            FFLOAT_DEFAULT => DataTypeRead {
                data_type: DataType::FFloat,
                little_endian,
            },
            GFLOAT_DEFAULT => DataTypeRead {
                data_type: DataType::GFloat,
                little_endian,
            },
            DFLOAT_DEFAULT => DataTypeRead {
                data_type: DataType::DFloat,
                little_endian,
            },
            STRING_NULL_TERM => DataTypeRead {
                data_type: DataType::StringNullTerm,
                little_endian,
            },
            BYTE_ARRAY => DataTypeRead {
                data_type: DataType::ByteArray,
                little_endian,
            },
            UNSIGNED_INT_BIGENDIAN => DataTypeRead {
                data_type: DataType::UnsignedInt,
                little_endian: false,
            },
            SIGNED_INT_BIGENDIAN => DataTypeRead {
                data_type: DataType::SignedInt,
                little_endian: false,
            },
            FLOAT32_BIGENDIAN => DataTypeRead {
                data_type: DataType::Float32,
                little_endian: false,
            },
            FLOAT64_BIGENDIAN => DataTypeRead {
                data_type: DataType::Float64,
                little_endian: false,
            },
            UNSIGNED_INT_LITTLEENDIAN => DataTypeRead {
                data_type: DataType::UnsignedInt,
                little_endian: true,
            },
            SIGNED_INT_LITTLEENDIAN => DataTypeRead {
                data_type: DataType::SignedInt,
                little_endian: true,
            },
            FLOAT32_INT_LITTLEENDIAN => DataTypeRead {
                data_type: DataType::Float32,
                little_endian: true,
            },
            FLOAT64_INT_LITTLEENDIAN => DataTypeRead {
                data_type: DataType::Float64,
                little_endian: true,
            },
            _ => {
                println!("Found data type: {}", datatype);
                panic!("Data type not found. Type was:")
            }
        };

        let value_range_valid = utils::read(stream, little_endian, &mut pos);
        let signal_min = utils::read(stream, little_endian, &mut pos);
        let signal_max = utils::read(stream, little_endian, &mut pos);
        let sample_rate = utils::read(stream, little_endian, &mut pos);
        let long_name = utils::read(stream, little_endian, &mut pos);
        let display_name = utils::read(stream, little_endian, &mut pos);
        let addition_byte_offset = utils::read(stream, little_endian, &mut pos);

        (
            CNBLOCK {
                block_type,
                block_size,
                next,
                conversion_formula,
                source_ext,
                dependency,
                comment,
                channel_type,
                short_name,
                desc,
                start_offset,
                bit_number,
                data_type,
                value_range_valid,
                signal_min,
                signal_max,
                sample_rate,
                long_name,
                display_name,
                addition_byte_offset,
            },
            pos,
        )
    }

    pub fn name(self, stream: &[u8], little_endian: bool) -> String {
        let mut name = "".to_string();

        if self.channel_type == 1 {
            name = "time".to_string();
        } else if self.long_name != 0 {
            let (tx, _pos) = TXBLOCK::read(stream, self.long_name as usize, little_endian);

            name = match std::str::from_utf8(&tx.text) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
        }

        name
    }
}

#[derive(Debug, Clone)]
pub struct CCBLOCK {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub physical_range_valid: u16,
    pub physical_min: f64,
    pub physical_max: f64,
    pub unit: [u8; 20],
    pub conversion_type: u16,
    pub size_info: u16,
    pub conversion_data: ConversionData,
}

impl CCBLOCK {
    pub fn read(stream: &[u8], little_endian: bool) -> (CCBLOCK, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream[position..position + 2].try_into().expect("msg");
        position += block_type.len();

        if !utils::eq(&block_type, &[b'C', b'C']) {
            panic!("CC not found");
        }

        let block_size: u16 = utils::read(stream, little_endian, &mut position);
        let physical_range_valid: u16 = utils::read(stream, little_endian, &mut position);
        let physical_min: f64 = utils::read(stream, little_endian, &mut position);
        let physical_max: f64 = utils::read(stream, little_endian, &mut position);
        let unit: [u8; 20] = stream[position..position + 20].try_into().expect("msg");
        position += unit.len();
        let conversion_type: u16 = utils::read(stream, little_endian, &mut position);
        let size_info: u16 = utils::read(stream, little_endian, &mut position);

        let datatype = 1;

        let (conversion_data, pos) = ConversionData::read(stream, little_endian, datatype);
        position += pos;

        (
            CCBLOCK {
                block_type,
                block_size,
                physical_range_valid,
                physical_min,
                physical_max,
                unit,
                conversion_type,
                size_info,
                conversion_data,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConversionData {
    Parameters,
    Table,
    Text,
}

impl ConversionData {
    pub fn read(_data: &[u8], _little_endian: bool, datatype: u8) -> (ConversionData, usize) {
        if datatype == 1 {
            (ConversionData::Parameters, 1)
        } else {
            (ConversionData::Table, 1)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Parameters {
    ConversionLinear,
    ConversionPoly,
    ConversionExponetial,
    ConversionLog,
    ConversionRational,
}

impl Parameters {
    pub fn read(_data: &[u8], _little_endian: bool) -> (Parameters, usize) {
        (Parameters::ConversionLinear, 10)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionLinear {
    pub p1: f64,
    pub p2: f64,
}

impl ConversionLinear {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionLinear, usize) {
        let mut position = 0;
        let p1 = utils::read(stream, little_endian, &mut position);
        let p2 = utils::read(stream, little_endian, &mut position);

        (ConversionLinear { p1, p2 }, position)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionPoly {
    pub p1: f64,
    pub p2: f64,
    pub p3: f64,
    pub p4: f64,
    pub p5: f64,
    pub p6: f64,
}

impl ConversionPoly {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionPoly, usize) {
        let mut position = 0;
        let p1: f64 = utils::read(stream, little_endian, &mut position);
        let p2: f64 = utils::read(stream, little_endian, &mut position);
        let p3: f64 = utils::read(stream, little_endian, &mut position);
        let p4: f64 = utils::read(stream, little_endian, &mut position);
        let p5: f64 = utils::read(stream, little_endian, &mut position);
        let p6: f64 = utils::read(stream, little_endian, &mut position);

        (
            ConversionPoly {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionExponetial {
    pub p1: f64,
    pub p2: f64,
    pub p3: f64,
    pub p4: f64,
    pub p5: f64,
    pub p6: f64,
    pub p7: f64,
}

impl ConversionExponetial {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionExponetial, usize) {
        let mut position = 0;
        let p1: f64 = utils::read(stream, little_endian, &mut position);
        let p2: f64 = utils::read(stream, little_endian, &mut position);
        let p3: f64 = utils::read(stream, little_endian, &mut position);
        let p4: f64 = utils::read(stream, little_endian, &mut position);
        let p5: f64 = utils::read(stream, little_endian, &mut position);
        let p6: f64 = utils::read(stream, little_endian, &mut position);
        let p7: f64 = utils::read(stream, little_endian, &mut position);

        (
            ConversionExponetial {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
                p7,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionLog {
    pub p1: f64,
    pub p2: f64,
    pub p3: f64,
    pub p4: f64,
    pub p5: f64,
    pub p6: f64,
    pub p7: f64,
}

impl ConversionLog {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionLog, usize) {
        let mut position = 0;
        let p1: f64 = utils::read(stream, little_endian, &mut position);
        let p2: f64 = utils::read(stream, little_endian, &mut position);
        let p3: f64 = utils::read(stream, little_endian, &mut position);
        let p4: f64 = utils::read(stream, little_endian, &mut position);
        let p5: f64 = utils::read(stream, little_endian, &mut position);
        let p6: f64 = utils::read(stream, little_endian, &mut position);
        let p7: f64 = utils::read(stream, little_endian, &mut position);

        (
            ConversionLog {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
                p7,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionRational {
    pub p1: f64,
    pub p2: f64,
    pub p3: f64,
    pub p4: f64,
    pub p5: f64,
    pub p6: f64,
}

impl ConversionRational {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionRational, usize) {
        let mut position = 0;
        let p1: f64 = utils::read(stream, little_endian, &mut position);
        let p2: f64 = utils::read(stream, little_endian, &mut position);
        let p3: f64 = utils::read(stream, little_endian, &mut position);
        let p4: f64 = utils::read(stream, little_endian, &mut position);
        let p5: f64 = utils::read(stream, little_endian, &mut position);
        let p6: f64 = utils::read(stream, little_endian, &mut position);

        (
            ConversionRational {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Table {
    ConversionTabular,
}

#[derive(Debug, Clone)]
pub struct ConversionTabular {
    pub value: Vec<TableEntry>,
}

impl ConversionTabular {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionTabular, usize) {
        let mut position = 0;
        let mut value = Vec::new();
        for _i in 0..1 {
            let (temp, pos) = TableEntry::read(stream, little_endian);
            position += pos;
            value.push(temp);
        }

        (ConversionTabular { value }, position)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TableEntry {
    pub internal: f64,
    pub physical: f64,
}

impl TableEntry {
    pub fn read(stream: &[u8], little_endian: bool) -> (TableEntry, usize) {
        let mut position = 0;
        let internal = utils::read(stream, little_endian, &mut position);
        let physical = utils::read(stream, little_endian, &mut position);

        (TableEntry { internal, physical }, position)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Text {
    ConversionTextFormula,
    ConversionTextRangeTable,
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionTextFormula {
    pub formula: [u8; 256],
}

impl ConversionTextFormula {
    pub fn read(stream: &[u8], _little_endian: bool) -> (ConversionTextFormula, usize) {
        let mut position = 0;
        let formula: [u8; 256] = stream.try_into().expect("msg");
        position += formula.len();

        (ConversionTextFormula { formula }, position)
    }
}

#[derive(Debug, Clone)]
pub struct ConversionTextTable {
    pub table: Vec<TextTableEntry>,
}

impl ConversionTextTable {
    pub fn read(stream: &[u8], little_endian: bool, number: usize) -> (ConversionTextTable, usize) {
        let mut position = 0;
        let mut table = Vec::new();
        for _i in 0..number - 1 {
            let (table_entry, pos) = TextTableEntry::read(stream, little_endian);
            table.push(table_entry);
            position += pos;
        }

        (ConversionTextTable { table }, position)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextTableEntry {
    pub internal: f64,
    pub text: [u8; 32],
}

impl TextTableEntry {
    pub fn read(stream: &[u8], little_endian: bool) -> (TextTableEntry, usize) {
        let mut position = 0;
        let internal = utils::read(stream, little_endian, &mut position);
        let text: [u8; 32] = stream.try_into().expect("msg");

        (TextTableEntry { internal, text }, position)
    }
}

#[derive(Debug, Clone)]
pub struct ConversionTextRangeTable {
    pub undef1: f64,
    pub undef2: f64,
    pub txblock: u32,
    pub entry: Vec<TextRange>,
}

impl ConversionTextRangeTable {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionTextRangeTable, usize) {
        let mut position = 0;
        let undef1 = utils::read(stream, little_endian, &mut position);
        let undef2 = utils::read(stream, little_endian, &mut position);
        let txblock = utils::read(stream, little_endian, &mut position);
        let entry = Vec::new();

        (
            ConversionTextRangeTable {
                undef1,
                undef2,
                txblock,
                entry,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextRange {
    pub lower: f64,
    pub upper: f64,
    pub txblock: u32,
}

impl TextRange {
    pub fn read(stream: &[u8], little_endian: bool) -> (TextRange, usize) {
        let mut position = 0;
        let lower = utils::read(stream, little_endian, &mut position);
        let upper = utils::read(stream, little_endian, &mut position);
        let txblock = utils::read(stream, little_endian, &mut position);

        (
            TextRange {
                lower,
                upper,
                txblock,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DateStruct {
    pub ms: u16,
    pub min: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u8,
}

impl DateStruct {
    pub fn read(stream: &[u8], little_endian: bool) -> (DateStruct, usize) {
        let mut position = 0;
        let ms = utils::read(stream, little_endian, &mut position);
        let min = utils::read(stream, little_endian, &mut position);
        let hour = utils::read(stream, little_endian, &mut position);
        let day = utils::read(stream, little_endian, &mut position);
        let month = utils::read(stream, little_endian, &mut position);
        let year = utils::read(stream, little_endian, &mut position);

        (
            DateStruct {
                ms,
                min,
                hour,
                day,
                month,
                year,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimeStruct {
    pub ms: u32,
    pub days: u8,
}

impl TimeStruct {
    pub fn read(stream: &[u8], little_endian: bool) -> (TimeStruct, usize) {
        let mut position = 0;
        let ms = utils::read(stream, little_endian, &mut position);
        let days = utils::read(stream, little_endian, &mut position);

        (TimeStruct { ms, days }, position)
    }
}

#[derive(Debug, Clone)]
pub struct CDBLOCK {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub dependency_type: u16,
    pub signal_number: u16,
    pub groups: Vec<Signals>,
    pub dims: Vec<u16>,
}

impl CDBLOCK {
    pub fn read(stream: &[u8], little_endian: bool) -> (CDBLOCK, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream[position..position + 2].try_into().expect("msg");
        position += block_type.len();
        let block_size: u16 = utils::read(stream, little_endian, &mut position);
        let dependency_type: u16 = utils::read(stream, little_endian, &mut position);
        let signal_number: u16 = utils::read(stream, little_endian, &mut position);

        let mut groups = Vec::with_capacity(signal_number as usize);

        for _i in 0..signal_number - 1 {
            let (temp, pos) = Signals::read(stream, little_endian);
            groups.push(temp);
            position += pos;
        }

        let mut dims = Vec::new();

        let no_dependencies = if dependency_type < 255 {
            dependency_type
        } else {
            dependency_type - 255
        };
        for _i in 0..no_dependencies - 1 {
            dims.push(utils::read(stream, little_endian, &mut position))
        }

        (
            CDBLOCK {
                block_type,
                block_size,
                dependency_type,
                signal_number,
                groups,
                dims,
            },
            position,
        )
    }
}

#[derive(Debug, Clone)]
pub struct Signals {
    pub data_group: u32,
    pub channel_group: u32,
    pub channel: u32,
}

impl Signals {
    pub fn read(stream: &[u8], little_endian: bool) -> (Self, usize) {
        let mut position = 0;
        let data_group = utils::read(stream, little_endian, &mut position);
        let channel_group = utils::read(stream, little_endian, &mut position);
        let channel = utils::read(stream, little_endian, &mut position);

        (
            Self {
                data_group,
                channel_group,
                channel,
            },
            position,
        )
    }
}

#[derive(Debug, Clone)]
pub struct CEBLOCK {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub extension_type: u16,
    pub additional: Vec<u8>,
}

impl CEBLOCK {
    pub fn read(stream: &[u8], little_endian: bool) -> (CEBLOCK, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream[position..position + 2].try_into().expect("msg");
        position += block_type.len();
        let block_size: u16 = utils::read(stream, little_endian, &mut position);
        let extension_type: u16 = utils::read(stream, little_endian, &mut position);

        let additional = stream[position..block_size as usize].to_vec();

        (
            CEBLOCK {
                block_type,
                block_size,
                extension_type,
                additional,
            },
            position,
        )
    }
}
