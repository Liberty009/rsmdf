use crate::utils::{self, FromBytes};
use itertools::izip;
use std::{convert::TryInto, mem};

// Define types from standard
type CHAR = u8;
type BYTE = u8;
//type UINT8 = u8;
type UINT16 = u16;
type INT16 = i16;
type UINT32 = u32;
type UINT64 = u64;
type BOOL = u16;
type REAL = f64;
type LINK = u32;

pub struct MDF3 {
    id: IDBLOCK,
    header: HDBLOCK,
    comment: TXBLOCK,
    data_groups: Vec<DGBLOCK>,
    little_endian: bool,
}

impl MDF3 {
    pub fn new(stream: &[u8]) -> Self {
        let (id, pos, little_endian) = IDBLOCK::read(stream);
        let (header, _pos) = HDBLOCK::read(&stream, pos, little_endian);
        let (comment, _pos) = TXBLOCK::read(stream, header.file_comment as usize, little_endian);

        MDF3 {
            id: id,
            header: header,
            comment: comment,
            data_groups: DGBLOCK::read_all(stream, little_endian, header.data_group_block as usize),
            little_endian,
        }
    }

    pub fn read_all(self, stream: &[u8]) {
        let mut channel_groups = Vec::new();
        for group in self.data_groups {
            channel_groups.append(&mut group.read_channel_groups(stream, self.little_endian));
        }

        let mut channels = Vec::new();
        for grp in channel_groups {
            channels.append(&mut grp.channels(stream, self.little_endian));
        }
    }
}

pub fn list(stream: &[u8]) -> Vec<DGBLOCK> {
    let (_id_block, position, little_endian) = IDBLOCK::read(stream);
    let (hd_block, _pos) = HDBLOCK::read(&stream, position, little_endian);
    //position += pos;

    let dg = DGBLOCK::read_all(stream, little_endian, hd_block.data_group_block as usize);
    return dg;
}

pub fn list_channels(stream: &[u8]) -> (Vec<CNBLOCK>, Vec<CGBLOCK>, Vec<DGBLOCK>) {
    let mut dg = Vec::new();
    let mut cg = Vec::new();
    let mut ch = Vec::new();

    let (_id_block, position, little_endian) = IDBLOCK::read(stream);
    let (hd_block, _pos) = HDBLOCK::read(&stream, position, little_endian);
    //position += pos;

    let mut next_dg = hd_block.data_group_block;

    while next_dg != 0 {
        let (dg_block, _position) = DGBLOCK::read(&stream, little_endian, next_dg as usize);
        next_dg = dg_block.next;
        let mut next_cg = dg_block.first;

        dg.push(dg_block);

        while next_cg != 0 {
            let (cg_block, _position) = CGBLOCK::read(stream, little_endian, next_cg as usize);
            next_cg = cg_block.next;
            let mut next_cn = cg_block.first;
            cg.push(cg_block);

            while next_cn != 0 {
                let (cn_block, _position) = CNBLOCK::read(stream, little_endian, next_cn as usize);
                next_cn = cn_block.next;

                ch.push(cn_block);
            }
        }
    }

    return (ch, cg, dg);
}

pub fn read(stream: &[u8], datagroup: &DGBLOCK, channel_grp: &CGBLOCK, channel: &CNBLOCK) {
    let channels: Vec<CNBLOCK> = channel_grp.channels(stream, true);
    let data_length = (channel_grp.record_number * channel_grp.record_size as u32) as usize;
    let data =
        &stream[datagroup.data_block as usize..(datagroup.data_block as usize + data_length)];

    let mut data_blocks = Vec::new();
    for i in 0..channel_grp.record_number {
        data_blocks.push(
            &data[(i * channel_grp.record_size as u32) as usize
                ..((i + 1) * channel_grp.record_size as u32) as usize],
        );
    }

    // let mut data_series = Vec::new();
    // for channel in channels {

    // 	data_series.push()
    // }

    let byte_offset = (channel.start_offset / 8) as usize;
    // let bit_offset = channel.start_offset % 8;

    let mut records = Vec::new();
    let mut pos = 0_usize;
    for _i in 0..channel_grp.record_number {
        records.push(&data[pos..pos + channel_grp.record_size as usize]);
        pos += channel_grp.record_size as usize;
    }

    // let mut time_raw: Vec<&[u8]> = Vec::new();
    // let mut some_raw: Vec<&[u8]> = Vec::new();

    // for record in records{
    // 	time_raw.push(&record[0..mem::size_of::<f64>() / mem::size_of::<u8>()]);
    // 	some_raw.push(&record[mem::size_of::<f64>() / mem::size_of::<u8>()..]);
    // }
    let little_endian = true;

    let mut time_raw = Vec::new();
    for rec in &records {
        time_raw.push(&rec[0..mem::size_of::<f64>() / mem::size_of::<u8>()])
    }
    let mut some_raw = Vec::new();
    for rec in &records {
        some_raw.push(&rec[byte_offset..])
    }

    let mut time: Vec<f64> = Vec::new();
    for t in time_raw {
        time.push(utils::read(t, little_endian, &mut 0));
    }
    let mut some: Vec<i8> = Vec::new();
    for s in some_raw {
        some.push(utils::read(s, little_endian, &mut 0));
    }

    for (t, s) in izip!(time, some) {
        println!("{}, {}", t, s);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IDBLOCK {
    pub file_id: [CHAR; 8],
    pub format_id: [CHAR; 8],
    pub program_id: [CHAR; 8],
    pub default_byte_order: UINT16,
    pub default_float_format: UINT16,
    pub version_number: UINT16,
    pub code_page_number: UINT16,
    pub reserved1: [CHAR; 2],
    pub reserved2: [CHAR; 30],
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
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub data_group_block: LINK,
    pub file_comment: LINK,
    pub program_block: LINK,
    pub data_group_number: UINT16,
    pub date: [CHAR; 10],
    pub time: [CHAR; 8],
    pub author: [CHAR; 32],
    pub department: [CHAR; 32],
    pub project: [CHAR; 32],
    pub subject: [CHAR; 32],
    pub timestamp: UINT64,
    pub utc_time_offset: INT16,
    pub time_quality: UINT16,
    pub timer_id: [CHAR; 32],
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
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub text: Vec<CHAR>,
}

impl TXBLOCK {
    pub fn read(stream: &[u8], position: usize, little_endian: bool) -> (TXBLOCK, usize) {
        let mut pos = position;

        let block_type: [u8; 2] = stream[pos..pos + 2].try_into().expect("");
        pos += 2;

        if !utils::eq(&block_type, &['T' as u8, 'X' as u8]) {
            panic!(
                "TXBLOCK type incorrect. Found : {}, {}",
                block_type[0], block_type[1]
            );
        }

        pos += block_type.len();
        let block_size = utils::read(&stream[pos..], little_endian, &mut pos);

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
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub program_data: Vec<CHAR>,
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
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub trigger_comment: LINK,
    pub trigger_events_number: UINT16,
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
    pub trigger_time: REAL,
    pub pre_trigger_time: REAL,
    pub post_trigger_time: REAL,
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
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub next: LINK,
    pub data_block: LINK,
    pub samples_reduced_number: UINT32,
    pub time_interval_length: REAL,
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
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub next: LINK,
    pub first: LINK,
    pub trigger_block: LINK,
    pub data_block: LINK,
    pub group_number: UINT16,
    pub id_number: UINT16,
    pub reserved: UINT32,
}

impl DGBLOCK {
    // Read the data stream in to a DGBLOCK type, return position reached
    pub fn read(stream: &[u8], little_endian: bool, position: usize) -> (Self, usize) {
        let mut pos = position;

        // Read block type to confirm
        let block_type: [u8; 2] = stream[pos..pos + 2].try_into().expect("msg");
        if !utils::eq(&block_type, &['D' as u8, 'G' as u8]) {
            panic!(
                "DGBLOCK not found. Found: {}, {}",
                block_type[0], block_type[1]
            );
        }

        pos += block_type.len();

        let block_size = utils::read(&stream, little_endian, &mut pos);
        let next = utils::read(&stream, little_endian, &mut pos);
        let first = utils::read(&stream, little_endian, &mut pos);
        let trigger_block = utils::read(&stream, little_endian, &mut pos);
        let data_block = utils::read(&stream, little_endian, &mut pos);
        let group_number = utils::read(&stream, little_endian, &mut pos);
        let id_number = utils::read(&stream, little_endian, &mut pos);
        let reserved = utils::read(&stream, little_endian, &mut pos);

        return (
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
            },
            pos,
        );
    }

    pub fn read_all(stream: &[u8], little_endian: bool, position: usize) -> Vec<Self> {
        let mut all = Vec::new();
        let mut next_dg = position;

        while next_dg != 0 {
            let (dg_block, _position) = DGBLOCK::read(&stream, little_endian, next_dg);
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
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub next: LINK,
    pub first: LINK,
    pub comment: LINK,
    pub record_id: UINT16,
    pub channel_number: UINT16,
    pub record_size: UINT16,
    pub record_number: UINT32,
    pub first_sample_reduction_block: LINK,
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
    data_type: DataType,
    little_endian: bool,
}

struct RecordedData<T> {
    record: Vec<T>,
}

impl<T: FromBytes> RecordedData<T> {
    pub fn new(stream: &[u8], datatype: DataTypeRead) -> Self {
        let mut record: Vec<T> = Vec::new();
        match datatype.data_type {
            DataType::UnsignedInt => {
                (record.push(utils::read(stream, datatype.little_endian, &mut 0)))
            }
            DataType::SignedInt => {
                (record.push(utils::read(stream, datatype.little_endian, &mut 0)))
            }
            DataType::Float32 => (record.push(utils::read(stream, datatype.little_endian, &mut 0))),
            DataType::Float64 => (record.push(utils::read(stream, datatype.little_endian, &mut 0))),
            DataType::StringNullTerm => {
                record.push(utils::from_be_bytes(stream))
            }
            DataType::ByteArray => (record.push(stream)),
            _ => (),
        }

        return RecordedData { record };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CNBLOCK {
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub next: LINK,
    pub conversion_formula: LINK,
    pub source_ext: LINK,
    pub dependency: LINK,
    pub comment: LINK,
    pub channel_type: UINT16,
    pub short_name: [CHAR; 32],
    pub desc: [CHAR; 128],
    pub start_offset: UINT16,
    pub bit_number: UINT16,
    pub data_type: DataTypeRead,
    pub value_range_valid: BOOL,
    pub signal_min: REAL,
    pub signal_max: REAL,
    pub sample_rate: REAL,
    pub long_name: LINK,
    pub display_name: LINK,
    pub addition_byte_offset: UINT16,
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
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub physical_range_valid: BOOL,
    pub physical_min: REAL,
    pub physical_max: REAL,
    pub unit: [CHAR; 20],
    pub conversion_type: UINT16,
    pub size_info: UINT16,
    pub conversion_data: ConversionData,
}

impl CCBLOCK {
    pub fn read(stream: &[u8], little_endian: bool) -> (CCBLOCK, usize) {
        let mut position = 0;
        let block_type: [CHAR; 2] = stream[position..position + 2].try_into().expect("msg");
        position += block_type.len();

        if !utils::eq(&block_type, &['C' as u8, 'C' as u8]) {
            panic!("CC not found");
        }

        let block_size: UINT16 = utils::read(&stream, little_endian, &mut position);
        let physical_range_valid: BOOL = utils::read(&stream, little_endian, &mut position);
        let physical_min: REAL = utils::read(&stream, little_endian, &mut position);
        let physical_max: REAL = utils::read(&stream, little_endian, &mut position);
        let unit: [CHAR; 20] = stream[position..position + 20].try_into().expect("msg");
        position += unit.len();
        let conversion_type: UINT16 = utils::read(&stream, little_endian, &mut position);
        let size_info: UINT16 = utils::read(&stream, little_endian, &mut position);

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
    pub p1: REAL,
    pub p2: REAL,
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
    pub p1: REAL,
    pub p2: REAL,
    pub p3: REAL,
    pub p4: REAL,
    pub p5: REAL,
    pub p6: REAL,
}

impl ConversionPoly {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionPoly, usize) {
        let mut position = 0;
        let p1: REAL = utils::read(&stream, little_endian, &mut position);
        let p2: REAL = utils::read(&stream, little_endian, &mut position);
        let p3: REAL = utils::read(&stream, little_endian, &mut position);
        let p4: REAL = utils::read(&stream, little_endian, &mut position);
        let p5: REAL = utils::read(&stream, little_endian, &mut position);
        let p6: REAL = utils::read(&stream, little_endian, &mut position);

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
    pub p1: REAL,
    pub p2: REAL,
    pub p3: REAL,
    pub p4: REAL,
    pub p5: REAL,
    pub p6: REAL,
    pub p7: REAL,
}

impl ConversionExponetial {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionExponetial, usize) {
        let mut position = 0;
        let p1: REAL = utils::read(&stream, little_endian, &mut position);
        let p2: REAL = utils::read(&stream, little_endian, &mut position);
        let p3: REAL = utils::read(&stream, little_endian, &mut position);
        let p4: REAL = utils::read(&stream, little_endian, &mut position);
        let p5: REAL = utils::read(&stream, little_endian, &mut position);
        let p6: REAL = utils::read(&stream, little_endian, &mut position);
        let p7: REAL = utils::read(&stream, little_endian, &mut position);

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
    pub p1: REAL,
    pub p2: REAL,
    pub p3: REAL,
    pub p4: REAL,
    pub p5: REAL,
    pub p6: REAL,
    pub p7: REAL,
}

impl ConversionLog {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionLog, usize) {
        let mut position = 0;
        let p1: REAL = utils::read(&stream, little_endian, &mut position);
        let p2: REAL = utils::read(&stream, little_endian, &mut position);
        let p3: REAL = utils::read(&stream, little_endian, &mut position);
        let p4: REAL = utils::read(&stream, little_endian, &mut position);
        let p5: REAL = utils::read(&stream, little_endian, &mut position);
        let p6: REAL = utils::read(&stream, little_endian, &mut position);
        let p7: REAL = utils::read(&stream, little_endian, &mut position);

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
    pub p1: REAL,
    pub p2: REAL,
    pub p3: REAL,
    pub p4: REAL,
    pub p5: REAL,
    pub p6: REAL,
}

impl ConversionRational {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionRational, usize) {
        let mut position = 0;
        let p1: REAL = utils::read(&stream, little_endian, &mut position);
        let p2: REAL = utils::read(&stream, little_endian, &mut position);
        let p3: REAL = utils::read(&stream, little_endian, &mut position);
        let p4: REAL = utils::read(&stream, little_endian, &mut position);
        let p5: REAL = utils::read(&stream, little_endian, &mut position);
        let p6: REAL = utils::read(&stream, little_endian, &mut position);

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
    pub internal: REAL,
    pub physical: REAL,
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
    pub formula: [CHAR; 256],
}

impl ConversionTextFormula {
    pub fn read(stream: &[u8], _little_endian: bool) -> (ConversionTextFormula, usize) {
        let mut position = 0;
        let formula: [CHAR; 256] = stream.try_into().expect("msg");
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
    pub internal: REAL,
    pub text: [CHAR; 32],
}

impl TextTableEntry {
    pub fn read(stream: &[u8], little_endian: bool) -> (TextTableEntry, usize) {
        let mut position = 0;
        let internal = utils::read(stream, little_endian, &mut position);
        let text: [CHAR; 32] = stream.try_into().expect("msg");

        return (TextTableEntry { internal, text }, position);
    }
}

#[derive(Debug, Clone)]
pub struct ConversionTextRangeTable {
    pub undef1: REAL,
    pub undef2: REAL,
    pub txblock: LINK,
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
    pub lower: REAL,
    pub upper: REAL,
    pub txblock: LINK,
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
    pub ms: UINT16,
    pub min: BYTE,
    pub hour: BYTE,
    pub day: BYTE,
    pub month: BYTE,
    pub year: BYTE,
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
    pub ms: UINT32,
    pub days: BYTE,
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
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub dependency_type: UINT16,
    pub signal_number: UINT16,
    pub groups: Vec<Signal>,
    pub dims: Vec<UINT16>,
}

impl CDBLOCK {
    pub fn read(stream: &[u8], little_endian: bool) -> (CDBLOCK, usize) {
        let mut position = 0;
        let block_type: [CHAR; 2] = stream[position..position + 2].try_into().expect("msg");
        position += block_type.len();
        let block_size: UINT16 = utils::read(&stream, little_endian, &mut position);
        let dependency_type: UINT16 = utils::read(&stream, little_endian, &mut position);
        let signal_number: UINT16 = utils::read(&stream, little_endian, &mut position);

        let mut groups = Vec::new();

        for _i in 0..signal_number - 1 {
            let (temp, pos) = Signal::read(&stream, little_endian);
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

#[derive(Debug, Clone, Copy)]
pub struct Signal {
    pub data_group: LINK,
    pub channel_group: LINK,
    pub channel: LINK,
}

impl Signal {
    pub fn read(stream: &[u8], little_endian: bool) -> (Signal, usize) {
        let mut position = 0;
        let data_group = utils::read(&stream, little_endian, &mut position);
        let channel_group = utils::read(&stream, little_endian, &mut position);
        let channel = utils::read(&stream, little_endian, &mut position);

        return (
            Signal {
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
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub extension_type: UINT16,
    pub additional: Vec<u8>,
}

impl CEBLOCK {
    pub fn read(stream: &[u8], little_endian: bool) -> (CEBLOCK, usize) {
        let mut position = 0;
        let block_type: [CHAR; 2] = stream[position..position + 2].try_into().expect("msg");
        position += block_type.len();
        let block_size: UINT16 = utils::read(&stream, little_endian, &mut position);
        let extension_type: UINT16 = utils::read(&stream, little_endian, &mut position);

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

// pub enum Supplement {
//     DIMBlock,
//     VectorBlock,
// }

// impl Supplement {
//     pub fn read(_stream: &[u8], _little_endian: bool) -> Supplement {
//         return Supplement::DIMBlock;
//     }
// }

// pub struct DIMBlock {
//     pub module_number: UINT16,
//     pub address: UINT32,
//     pub desc: [CHAR; 80],
//     pub ecu_id: [CHAR; 32],
// }

// impl DIMBlock {
//     pub fn read(stream: &[u8], little_endian: bool) -> (DIMBlock, usize) {
//         let mut position = 0;
//         let module_number: UINT16 =
//             utils::read(&stream[position..], little_endian, &mut position);
//         let address: UINT32 = utils::read(&stream[position..], little_endian, &mut position);
//         let desc: [CHAR; 80] = stream[position..position+80].try_into().expect("msg");
//         position += desc.len();
//         let ecu_id: [CHAR; 32] = stream[position..position+32].try_into().expect("msg");
//         position += ecu_id.len();

//         return (
//             DIMBlock {
//                 module_number,
//                 address,
//                 desc,
//                 ecu_id,
//             },
//             position,
//         );
//     }
// }

// pub struct VectorBlock {
//     pub can_id: UINT32,
//     pub can_channel: UINT32,
//     pub message_name: [CHAR; 36],
//     pub sender_name: [CHAR; 36],
// }

// impl VectorBlock {
//     pub fn read(stream: &[u8], little_endian: bool) -> (VectorBlock, usize) {
//         let mut position = 0;
//         let can_id: UINT32 = utils::read(&stream[position..], little_endian, &mut position);
//         let can_channel: UINT32 =
//             utils::read(&stream[position..], little_endian, &mut position);
//         let message_name: [CHAR; 36] = stream[position..].try_into().expect("msg");
//         position += message_name.len();
//         let sender_name: [CHAR; 36] = stream[position..].try_into().expect("msg");
//         position += sender_name.len();

//         return (
//             VectorBlock {
//                 can_id,
//                 can_channel,
//                 message_name,
//                 sender_name,
//             },
//             position,
//         );
//     }
// }
