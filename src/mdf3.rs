use crate::{mdf, signal::Signal};
use crate::{signal, utils};
use std::fs::File;
use std::io::prelude::*;
use std::{convert::TryInto, mem};

#[derive(Debug, Clone)]
pub (crate) struct MDF3 {
    id: IDBLOCK,
    header: HDBLOCK,
    comment: TXBLOCK,
    data_groups: Vec<DGBLOCK>,
    channels: Vec<CNBLOCK>,
    channel_groups: Vec<CGBLOCK>,
    little_endian: bool,
    file: Vec<u8>,
}

impl mdf::MDFFile for MDF3 {
    fn new(filepath: &str) -> Self {
        let mut file = File::open(filepath).expect("Could not read file");
        let mut stream = Vec::new();
        let _ = file.read_to_end(&mut stream);
        let (id, pos, little_endian) = IDBLOCK::read(&stream);
        let (header, _pos) = HDBLOCK::read(&stream, pos, little_endian);
        let (comment, _pos) = TXBLOCK::read(&stream, header.file_comment as usize, little_endian);
        let mut mdf = MDF3 {
            id: id,
            header: header,
            comment: comment,
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

        return mdf;
    }

    fn read_all(&mut self) {
        let mut channel_groups = Vec::new();
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

        // return (ch, cg, dg);
    }

    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> signal::Signal {
        let channels: Vec<CNBLOCK> = self.channel_groups[channel_grp].channels(&self.file, true);
        let data_length = (&self.channel_groups[channel_grp].record_number
            * self.channel_groups[channel_grp].record_size as u32)
            as usize;
        let data = &self.file[self.data_groups[datagroup].data_block as usize
            ..(self.data_groups[datagroup].data_block as usize + data_length)];

        let mut data_blocks = Vec::new();
        for i in 0..self.channel_groups[channel_grp].record_number {
            data_blocks.push(
                &data[(i * self.channel_groups[channel_grp].record_size as u32) as usize
                    ..((i + 1) * self.channel_groups[channel_grp].record_size as u32) as usize],
            );
        }

        let byte_offset = (self.channels[channel].start_offset / 8) as usize;
        let _bit_offset = self.channels[channel].start_offset % 8;

        let mut records = Vec::new();
        let mut pos = 0_usize;
        for _i in 0..self.channel_groups[channel_grp].record_number {
            records.push(&data[pos..pos + self.channel_groups[channel_grp].record_size as usize]);
            pos += self.channel_groups[channel_grp].record_size as usize;
        }

        let mut time_raw = Vec::new();
        for rec in &records {
            time_raw.push(&rec[0..channels[0].data_type.len()])
        }
        let mut some_raw = Vec::new();
        let end = byte_offset + channels[1].data_type.len();
        for rec in &records {
            some_raw.push(&rec[byte_offset..end])
        }

        let mut time = Vec::new();
        for raw in time_raw {
            time.push(Record::new(raw, channels[0].data_type));
        }

        let mut some = Vec::new();
        for raw in some_raw {
            some.push(Record::new(raw, channels[1].data_type));
        }

        return signal::Signal::new(
            time.iter().map(|x| x.extract()).collect(),
            some.iter().map(|x| x.extract()).collect(),
            "Unit".to_string(),
            "Measurement".to_string(),
            "This is some measurement".to_string(),
            false,
        );
    }

    fn cut(&self, start: f64, end: f64, include_ends: bool, time_from_zero: bool) {
        let delta = if time_from_zero { start } else { 0.0 };
    }

    fn export(&self, format: &str, filename: &str) {}
    fn filter(&self, channels: &str) {}
    fn resample(&self, raster: mdf::RasterType, version: &str, time_from_zero: bool) -> Self {
        return self.clone();
    }
    fn select(
        &self,
        channels: mdf::ChannelsType,
        record_offset: isize,
        raw: bool,
        copy_master: bool,
        ignore_value2text_conversions: bool,
        record_count: isize,
        validate: bool,
    ) -> Vec<Signal> {
        return Vec::new();
    }
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

        let default_byte_order = utils::read(&stream, true, &mut position);

        let little_endian = if default_byte_order == 0 { true } else { false };

        let default_float_format = utils::read(&stream, little_endian, &mut position);

        let version_number = utils::read(&stream, little_endian, &mut position);

        let code_page_number = utils::read(&stream, little_endian, &mut position);

        let reserved1: [u8; 2] = [stream[position], stream[position + 1]];
        position += reserved1.len();
        let reserved2: [u8; 30] = stream[position..position + 30].try_into().expect("msg");
        position += reserved2.len();

        return (
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
        );
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

        if !utils::eq(&block_type, &['H' as u8, 'D' as u8]) {
            panic!("Incorrect type for HDBLOCK");
        }

        pos += block_type.len();
        let block_size = utils::read(&stream, little_endian, &mut pos);
        let data_group_block = utils::read(&stream, little_endian, &mut pos);
        let file_comment = utils::read(&stream, little_endian, &mut pos);
        let program_block = utils::read(&stream, little_endian, &mut pos);
        let data_group_number = utils::read(&stream, little_endian, &mut pos);
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
        let timestamp = utils::read(&stream, little_endian, &mut pos);
        let utc_time_offset = utils::read(&stream, little_endian, &mut pos);
        let time_quality = utils::read(&stream, little_endian, &mut pos);
        let timer_id: [u8; 32] = stream[pos..pos + 32].try_into().expect("msg");
        pos += timer_id.len();

        return (
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
        );
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
        if !utils::eq(&block_type, &['T' as u8, 'X' as u8]) {
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

        return (
            TXBLOCK {
                block_type,
                block_size,
                text,
            },
            pos,
        );
    }

    pub fn name(self) -> String {
        //let mut name = "".to_string();

        //let (tx, _pos) = Self::read(stream, little_endian);

        let name = utils::extract_name(&self.text);

        return name;
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
        if !utils::eq(&block_type, &['P' as u8, 'R' as u8]) {
            panic!("PR Block not found");
        }

        pos += block_type.len();

        let block_size = utils::read(&stream, little_endian, &mut pos);
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

        return (
            PRBLOCK {
                block_type,
                block_size,
                program_data,
            },
            pos,
        );
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
        if !utils::eq(&block_type, &['T' as u8, 'R' as u8]) {
            panic!(
                "TRBLOCK not found. Found: {}, {}",
                block_type[0], block_type[1]
            );
        }

        pos += block_type.len();

        let block_size = utils::read(&stream[pos..], little_endian, &mut pos);
        let trigger_comment = utils::read(&stream[pos..], little_endian, &mut pos);
        let trigger_events_number = utils::read(&stream, little_endian, &mut pos);
        let (events, pos) =
            TRBLOCK::read_events(&stream, pos, little_endian, trigger_events_number);

        return (
            TRBLOCK {
                block_type,
                block_size,
                trigger_comment,
                trigger_events_number,
                events,
            },
            pos,
        );
    }

    fn read_events(
        stream: &[u8],
        position: usize,
        little_endian: bool,
        no_events: u16,
    ) -> (Vec<Event>, usize) {
        let mut events = Vec::new();
        let mut pos1 = position;
        for _i in 0..no_events {
            let (event, pos) = Event::read(&stream, pos1, little_endian);
            events.push(event);
            pos1 += pos;
        }

        return (events, position);
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
        let trigger_time = utils::read(&stream, little_endian, &mut pos);
        let pre_trigger_time = utils::read(&stream, little_endian, &mut pos);
        let post_trigger_time = utils::read(&stream, little_endian, &mut pos);
        return (
            Event {
                trigger_time,
                pre_trigger_time,
                post_trigger_time,
            },
            position,
        );
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
        let block_size = utils::read(&stream, little_endian, &mut position);
        let next = utils::read(&stream, little_endian, &mut position);
        let data_block = utils::read(&stream, little_endian, &mut position);
        let samples_reduced_number = utils::read(&stream, little_endian, &mut position);
        let time_interval_length = utils::read(&stream, little_endian, &mut position);

        return (
            SRBLOCK {
                block_type,
                block_size,
                next,
                data_block,
                samples_reduced_number,
                time_interval_length,
            },
            position,
        );
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
    // Read the data stream in to a DGBLOCK type, return position reached
    pub fn read(stream: &[u8], little_endian: bool, position: &mut usize) -> Self {
        let mut pos = position;

        // Read block type to confirm
        let block_type: [u8; 2] = stream[*pos..*pos + 2].try_into().expect("msg");
        if !utils::eq(&block_type, &['D' as u8, 'G' as u8]) {
            panic!(
                "DGBLOCK not found. Found: {}, {}",
                block_type[0], block_type[1]
            );
        }

        *pos += block_type.len();

        let block_size = utils::read(&stream, little_endian, &mut pos);
        let next = utils::read(&stream, little_endian, &mut pos);
        let first = utils::read(&stream, little_endian, &mut pos);
        let trigger_block = utils::read(&stream, little_endian, &mut pos);
        let data_block = utils::read(&stream, little_endian, &mut pos);
        let group_number = utils::read(&stream, little_endian, &mut pos);
        let id_number = utils::read(&stream, little_endian, &mut pos);
        let reserved = utils::read(&stream, little_endian, &mut pos);

        return DGBLOCK {
            block_type,
            block_size,
            next,
            first,
            trigger_block,
            data_block,
            group_number,
            id_number,
            reserved,
        };
    }

    pub fn read_all(stream: &[u8], little_endian: bool, position: usize) -> Vec<Self> {
        let mut all = Vec::new();
        let mut next_dg = position;

        while next_dg != 0 {
            let dg_block = DGBLOCK::read(&stream, little_endian, &mut next_dg);
            next_dg = dg_block.next as usize;
            all.push(dg_block);
        }

        return all;
    }

    pub fn read_channel_groups(self, stream: &[u8], little_endian: bool) -> Vec<CGBLOCK> {
        let mut channel_grps = Vec::new();
        let mut next = self.first as usize;
        while next != 0 {
            let (cg_block, _pos) = CGBLOCK::read(stream, little_endian, next);
            next = cg_block.next as usize;
            channel_grps.push(cg_block);
        }
        return channel_grps;
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

        if !utils::eq(&block_type, &['C' as u8, 'G' as u8]) {
            panic!(
                "CGBLOCK not found. Found: {}, {}",
                block_type[0] as char, block_type[1] as char
            );
        }

        pos += block_type.len();

        let block_size = utils::read(&stream, little_endian, &mut pos);
        let next = utils::read(&stream, little_endian, &mut pos);
        let first = utils::read(&stream, little_endian, &mut pos);
        let comment = utils::read(&stream, little_endian, &mut pos);
        let record_id = utils::read(&stream, little_endian, &mut pos);
        let channel_number = utils::read(&stream, little_endian, &mut pos);
        let record_size = utils::read(&stream, little_endian, &mut pos);
        let record_number = utils::read(&stream, little_endian, &mut pos);
        let first_sample_reduction_block = utils::read(&stream, little_endian, &mut pos);

        return (
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
        );
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

        return ch;
    }
}

// pub enum Types{
// 	u8,
// 	u16,
// 	u32,
// 	u64,
// 	i8,
// 	i16,
// 	i32,
// 	i64,
// }

// pub struct DataType {
// 	pub t: Types,

// }

// fn read_record<T: utils::FromBytes>(value: Record) -> T {
// 	let val: T = match value {
// 		Record::Uint(number) => number,
// 		Record::Int(number) => number,
// 		Record::Float32(number) => number,
// 		Record::Float64(number) => number,
// 		_ => panic!("Help!")
// 	};

// 	return val;
// }

pub fn print_record(value: Record) {
    match value {
        Record::Uint(number) => print!("{}", number),
        Record::Int(number) => print!("{}", number),
        Record::Float32(number) => print!("{}", number),
        Record::Float64(number) => print!("{}", number),
        // _ => panic!("Help!")
    };
}

pub enum Record {
    Uint(u8),
    Int(i8),
    Float32(f32),
    Float64(f64),
}

impl Record {
    pub fn new(stream: &[u8], dtype: DataTypeRead) -> Self {
        let rec = match dtype.data_type {
            DataType::UnsignedInt => Self::unsigned_int(stream, dtype),
            DataType::SignedInt => Self::signed_int(stream, dtype),
            DataType::Float32 => Self::float32(stream, dtype),
            DataType::Float64 => Self::float64(stream, dtype),
            _ => (panic!("Incorrect or not implemented type!")),
        };

        return rec;
    }

    pub fn extract(&self) -> f64 {
        let value = match self {
            Record::Uint(number) => *number as f64,
            Record::Int(number) => *number as f64,
            Record::Float32(number) => *number as f64,
            Record::Float64(number) => *number as f64,
            // _ => panic!("Help!")
        };

        return value;
    }

    fn unsigned_int(stream: &[u8], dtype: DataTypeRead) -> Self {
        let records = utils::read(stream, dtype.little_endian, &mut 0);

        return Self::Uint(records);
    }

    fn signed_int(stream: &[u8], dtype: DataTypeRead) -> Self {
        let records = utils::read(stream, dtype.little_endian, &mut 0);

        return Self::Int(records);
    }

    fn float32(stream: &[u8], dtype: DataTypeRead) -> Self {
        let records = utils::read(stream, dtype.little_endian, &mut 0);

        return Self::Float32(records);
    }
    fn float64(stream: &[u8], dtype: DataTypeRead) -> Self {
        let records = utils::read(stream, dtype.little_endian, &mut 0);

        return Self::Float64(records);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DataType {
    UnsignedInt,
    SignedInt,
    Float32,
    Float64,
    FFloat,
    GFloat,
    DFloat,
    StringNullTerm,
    ByteArray,
}

#[derive(Debug, Clone, Copy)]
pub struct DataTypeRead {
    pub data_type: DataType,
    pub little_endian: bool,
}

impl DataTypeRead {
    fn len(self) -> usize {
        let length = match self.data_type {
            DataType::UnsignedInt => mem::size_of::<u8>() / mem::size_of::<u8>(),
            DataType::SignedInt => mem::size_of::<i8>() / mem::size_of::<u8>(),
            DataType::Float32 => mem::size_of::<f32>() / mem::size_of::<u8>(),
            DataType::Float64 => mem::size_of::<f64>() / mem::size_of::<u8>(),
            DataType::FFloat => 0,
            DataType::GFloat => 0,
            DataType::DFloat => 0,
            DataType::StringNullTerm => 0,
            DataType::ByteArray => 0,
            // _ => panic!("")
        };
        return length;
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

//         return RecordedData{ data: result};
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

//         return Number { data: converted };
//     }
// }
// struct StringNull {
//     StringNull: String,
// }
// impl StringNull {
//     fn new() -> Self {
//         return StringNull {
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
        if !utils::eq(&block_type, &['C' as u8, 'N' as u8]) {
            panic!("CNBLOCK not found.");
        }

        let block_size = utils::read(&stream, little_endian, &mut pos);
        let next = utils::read(&stream, little_endian, &mut pos);
        let conversion_formula = utils::read(&stream, little_endian, &mut pos);
        let source_ext = utils::read(&stream, little_endian, &mut pos);
        let dependency = utils::read(&stream, little_endian, &mut pos);
        let comment = utils::read(&stream, little_endian, &mut pos);
        let channel_type = utils::read(&stream, little_endian, &mut pos);

        let short_name: [u8; 32] = stream[pos..pos + 32].try_into().expect("msg");
        pos += short_name.len();

        let desc: [u8; 128] = stream[pos..pos + 128].try_into().expect("msg");
        pos += desc.len();

        let start_offset = utils::read(&stream, little_endian, &mut pos);
        let bit_number = utils::read(&stream, little_endian, &mut pos);

        let datatype: u16 = utils::read(&stream, little_endian, &mut pos);
        let data_type = match datatype {
            0 => DataTypeRead {
                data_type: DataType::UnsignedInt,
                little_endian,
            },
            1 => DataTypeRead {
                data_type: DataType::SignedInt,
                little_endian: little_endian,
            },
            2 => DataTypeRead {
                data_type: DataType::Float32,
                little_endian: little_endian,
            },
            3 => DataTypeRead {
                data_type: DataType::Float64,
                little_endian: little_endian,
            },
            4 => DataTypeRead {
                data_type: DataType::FFloat,
                little_endian: little_endian,
            },
            5 => DataTypeRead {
                data_type: DataType::GFloat,
                little_endian: little_endian,
            },
            6 => DataTypeRead {
                data_type: DataType::DFloat,
                little_endian: little_endian,
            },
            7 => DataTypeRead {
                data_type: DataType::StringNullTerm,
                little_endian: little_endian,
            },
            8 => DataTypeRead {
                data_type: DataType::ByteArray,
                little_endian: little_endian,
            },
            9 => DataTypeRead {
                data_type: DataType::UnsignedInt,
                little_endian: false,
            },
            10 => DataTypeRead {
                data_type: DataType::SignedInt,
                little_endian: false,
            },
            11 => DataTypeRead {
                data_type: DataType::Float32,
                little_endian: false,
            },
            12 => DataTypeRead {
                data_type: DataType::Float64,
                little_endian: false,
            },
            13 => DataTypeRead {
                data_type: DataType::UnsignedInt,
                little_endian: true,
            },
            14 => DataTypeRead {
                data_type: DataType::SignedInt,
                little_endian: true,
            },
            15 => DataTypeRead {
                data_type: DataType::Float32,
                little_endian: true,
            },
            16 => DataTypeRead {
                data_type: DataType::Float64,
                little_endian: true,
            },
            _ => {
                println!("Found data type: {}", datatype);
                panic!("Data type not found. Type was:")
            }
        };

        let value_range_valid = utils::read(&stream, little_endian, &mut pos);
        let signal_min = utils::read(&stream, little_endian, &mut pos);
        let signal_max = utils::read(&stream, little_endian, &mut pos);
        let sample_rate = utils::read(&stream, little_endian, &mut pos);
        let long_name = utils::read(&stream, little_endian, &mut pos);
        let display_name = utils::read(&stream, little_endian, &mut pos);
        let addition_byte_offset = utils::read(&stream, little_endian, &mut pos);

        return (
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
        );
    }

    pub fn name(self, stream: &[u8], little_endian: bool) -> String {
        let mut name = "".to_string();

        if self.channel_type == 1 {
            name = "time".to_string();
        } else if self.comment != 0 {
            let (tx, _pos) = TXBLOCK::read(&stream, self.comment as usize, little_endian);

            name = tx.name();
        }

        return name;
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

        if !utils::eq(&block_type, &['C' as u8, 'C' as u8]) {
            panic!("CC not found");
        }

        let block_size: u16 = utils::read(&stream, little_endian, &mut position);
        let physical_range_valid: u16 = utils::read(&stream, little_endian, &mut position);
        let physical_min: f64 = utils::read(&stream, little_endian, &mut position);
        let physical_max: f64 = utils::read(&stream, little_endian, &mut position);
        let unit: [u8; 20] = stream[position..position + 20].try_into().expect("msg");
        position += unit.len();
        let conversion_type: u16 = utils::read(&stream, little_endian, &mut position);
        let size_info: u16 = utils::read(&stream, little_endian, &mut position);

        let datatype = 1;

        let (conversion_data, pos) = ConversionData::read(&stream, little_endian, datatype);
        position += pos;

        return (
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
        );
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
            return (ConversionData::Parameters, 1);
        } else {
            return (ConversionData::Table, 1);
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
        return (Parameters::ConversionLinear, 10);
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
        let p2 = utils::read(&stream, little_endian, &mut position);

        return (ConversionLinear { p1, p2 }, position);
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
        let p1: f64 = utils::read(&stream, little_endian, &mut position);
        let p2: f64 = utils::read(&stream, little_endian, &mut position);
        let p3: f64 = utils::read(&stream, little_endian, &mut position);
        let p4: f64 = utils::read(&stream, little_endian, &mut position);
        let p5: f64 = utils::read(&stream, little_endian, &mut position);
        let p6: f64 = utils::read(&stream, little_endian, &mut position);

        return (
            ConversionPoly {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
            },
            position,
        );
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
        let p1: f64 = utils::read(&stream, little_endian, &mut position);
        let p2: f64 = utils::read(&stream, little_endian, &mut position);
        let p3: f64 = utils::read(&stream, little_endian, &mut position);
        let p4: f64 = utils::read(&stream, little_endian, &mut position);
        let p5: f64 = utils::read(&stream, little_endian, &mut position);
        let p6: f64 = utils::read(&stream, little_endian, &mut position);
        let p7: f64 = utils::read(&stream, little_endian, &mut position);

        return (
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
        );
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
        let p1: f64 = utils::read(&stream, little_endian, &mut position);
        let p2: f64 = utils::read(&stream, little_endian, &mut position);
        let p3: f64 = utils::read(&stream, little_endian, &mut position);
        let p4: f64 = utils::read(&stream, little_endian, &mut position);
        let p5: f64 = utils::read(&stream, little_endian, &mut position);
        let p6: f64 = utils::read(&stream, little_endian, &mut position);
        let p7: f64 = utils::read(&stream, little_endian, &mut position);

        return (
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
        );
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
        let p1: f64 = utils::read(&stream, little_endian, &mut position);
        let p2: f64 = utils::read(&stream, little_endian, &mut position);
        let p3: f64 = utils::read(&stream, little_endian, &mut position);
        let p4: f64 = utils::read(&stream, little_endian, &mut position);
        let p5: f64 = utils::read(&stream, little_endian, &mut position);
        let p6: f64 = utils::read(&stream, little_endian, &mut position);

        return (
            ConversionRational {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
            },
            position,
        );
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
            let (temp, pos) = TableEntry::read(&stream, little_endian);
            position += pos;
            value.push(temp);
        }

        return (ConversionTabular { value }, position);
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
        let internal = utils::read(&stream, little_endian, &mut position);
        let physical = utils::read(&stream, little_endian, &mut position);

        return (TableEntry { internal, physical }, position);
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

        return (ConversionTextFormula { formula }, position);
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
            let (table_entry, pos) = TextTableEntry::read(&stream, little_endian);
            table.push(table_entry);
            position += pos;
        }

        return (ConversionTextTable { table }, position);
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

        return (TextTableEntry { internal, text }, position);
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
        let undef1 = utils::read(&stream, little_endian, &mut position);
        let undef2 = utils::read(&stream, little_endian, &mut position);
        let txblock = utils::read(&stream, little_endian, &mut position);
        let entry = Vec::new();

        return (
            ConversionTextRangeTable {
                undef1,
                undef2,
                txblock,
                entry,
            },
            position,
        );
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
        let lower = utils::read(&stream, little_endian, &mut position);
        let upper = utils::read(&stream, little_endian, &mut position);
        let txblock = utils::read(&stream, little_endian, &mut position);

        return (
            TextRange {
                lower,
                upper,
                txblock,
            },
            position,
        );
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
        let ms = utils::read(&stream, little_endian, &mut position);
        let min = utils::read(&stream, little_endian, &mut position);
        let hour = utils::read(&stream, little_endian, &mut position);
        let day = utils::read(&stream, little_endian, &mut position);
        let month = utils::read(&stream, little_endian, &mut position);
        let year = utils::read(&stream, little_endian, &mut position);

        return (
            DateStruct {
                ms,
                min,
                hour,
                day,
                month,
                year,
            },
            position,
        );
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
        let ms = utils::read(&stream, little_endian, &mut position);
        let days = utils::read(&stream, little_endian, &mut position);

        return (TimeStruct { ms, days }, position);
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
        let block_size: u16 = utils::read(&stream, little_endian, &mut position);
        let dependency_type: u16 = utils::read(&stream, little_endian, &mut position);
        let signal_number: u16 = utils::read(&stream, little_endian, &mut position);

        let mut groups = Vec::new();

        for _i in 0..signal_number - 1 {
            let (temp, pos) = Signals::read(&stream, little_endian);
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
            dims.push(utils::read(&stream, little_endian, &mut position))
        }

        return (
            CDBLOCK {
                block_type,
                block_size,
                dependency_type,
                signal_number,
                groups,
                dims,
            },
            position,
        );
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
        let data_group = utils::read(&stream, little_endian, &mut position);
        let channel_group = utils::read(&stream, little_endian, &mut position);
        let channel = utils::read(&stream, little_endian, &mut position);

        return (
            Self {
                data_group,
                channel_group,
                channel,
            },
            position,
        );
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
        let block_size: u16 = utils::read(&stream, little_endian, &mut position);
        let extension_type: u16 = utils::read(&stream, little_endian, &mut position);

        let additional = stream[position..block_size as usize].to_vec();

        return (
            CEBLOCK {
                block_type,
                block_size,
                extension_type,
                additional,
            },
            position,
        );
    }
}
