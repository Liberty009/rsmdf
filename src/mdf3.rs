use crate::utils;
use std::{convert::TryInto};


use byteorder::{BigEndian, ByteOrder, LittleEndian};

// Define types from standard
type CHAR = u8;
type BYTE = u8;
type UINT8 = u8;
type UINT16 = u16;
type INT16 = i16;
type UINT32 = u32;
type UINT64 = u64;
type BOOL = u16;
type REAL = f64;
type LINK = u32;

pub fn read(file: &[u8]) -> (IDBLOCK, bool, usize) {
	let (id_block, position, little_endian) = IDBLOCK::read(file);
	return (id_block, little_endian, position);
}

pub fn read_head(file: &[u8], little_endian: bool) -> (HDBLOCK, usize) {
	let (HDBLOCK, position) = HDBLOCK::read(file, little_endian);
	return (HDBLOCK, position)
}


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

fn eq(array1: &[u8], other: &[u8]) -> bool {
	array1.iter().zip(other.iter()).all(|(a,b)| a == b) 
}

impl IDBLOCK {
    pub fn read(stream: &[u8]) -> (IDBLOCK, usize, bool) {
        let file_id: [u8;8] = stream[0..8].try_into().expect("msg");
		if !eq(&file_id[..], &[0x4D, 0x44, 0x46, 0x20, 0x20, 0x20, 0x20, 0x20,]) {
			panic!("Error: Incorrect file type");
		}

        let format_id = stream[9..17].try_into().expect("msg");
        let program_id = stream[18..26].try_into().expect("msg");

        let default_byte_order = LittleEndian::read_u16(&stream[24..]);

		let little_endian = if default_byte_order == 0 {
			true
		} else {
			false
		};

        let default_float_format = if little_endian {
            LittleEndian::read_u16(&stream[26..])
        } else {
            BigEndian::read_u16(&stream[26..])
        };

        let version_number = if little_endian {
            LittleEndian::read_u16(&stream[28..])
        } else {
            BigEndian::read_u16(&stream[28..])
        };

        let code_page_number = if little_endian {
            LittleEndian::read_u16(&stream[30..])
        } else {
            BigEndian::read_u16(&stream[30..])
        };

        let reserved1 = [stream[32], stream[33]];
        let reserved2 = stream[34..].try_into().expect("msg");

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
            64,
			little_endian,
        );
    }
}

pub struct HDBLOCK {
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
    pub fn read(stream: &[u8], little_endian: bool) -> (HDBLOCK, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream[0..2].try_into().expect("");

		if !eq(&block_type, &['H' as u8, 'D' as u8]) {
			panic!("Incorrect type for HDBLOCK");
		}

        position += block_type.len();
        let block_size = utils::read_u16(&stream[position..], little_endian, &mut position);
        let data_group_block = utils::read_u32(&stream[position..], little_endian, &mut position);
        let file_comment = utils::read_u32(&stream[position..], little_endian, &mut position);
        let program_block = utils::read_u32(&stream[position..], little_endian, &mut position);
        let data_group_number = utils::read_u16(&stream[position..], little_endian, &mut position);
        let date: [u8; 10] = stream[position..position+10].try_into().expect("msg");
        position += date.len();
        let time: [u8; 8] = stream[position..position+8].try_into().expect("msg");
        position += time.len();
        let author: [u8; 32] = stream[position..position+32].try_into().expect("msg");
        position += author.len();
        let department: [u8; 32] = stream[position..position+32].try_into().expect("msg");
        position += department.len();
        let project: [u8; 32] = stream[position..position+32].try_into().expect("msg");
        position += project.len();
        let subject: [u8; 32] = stream[position..position+32].try_into().expect("msg");
        position += subject.len();
        let timestamp = utils::read_u64(&stream[position..], little_endian, &mut position);
        let utc_time_offset = utils::read_i16(&stream[position..], little_endian, &mut position);
        let time_quality = utils::read_u16(&stream[position..], little_endian, &mut position);
        let timer_id: [u8; 32] = stream[position..position+32].try_into().expect("msg");
        position += timer_id.len();

        return (
            HDBLOCK {
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
            position,
        );
    }
}

pub struct TXBLOCK {
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub text: Vec<CHAR>,
}

impl TXBLOCK {
    pub fn read(stream: &[u8], little_endian: bool) -> (TXBLOCK, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream[0..2].try_into().expect("");
        position += block_type.len();
        let block_size = utils::read_u16(&stream[position..], little_endian, &mut position);

        let mut text: Vec<u8> = vec![0; block_size as usize];
        text = stream[position..position+block_size as usize-5].try_into().expect("msg");
		
		// make sure that the text is utf8
		for c in &mut text {
			if 128 < *c {
				*c = 32;
			}
		}
        position += text.len();

        return (
            TXBLOCK {
                block_type,
                block_size,
                text,
            },
            position,
        );
    }
}

pub struct PRBLOCK {
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub program_data: Vec<CHAR>,
}

impl PRBLOCK {
    pub fn read(stream: &[u8], little_endian: bool) -> (PRBLOCK, usize) {
        let block_type:[u8; 2] = stream[0..2].try_into().expect("");
		if !eq(&block_type, &['P' as u8, 'R' as u8,]) {
			panic!("PR Block not found");
		}
        let block_size = if little_endian {
            LittleEndian::read_u16(&stream[2..])
        } else {
            BigEndian::read_u16(&stream[2..])
        };

        let mut program_data = vec![0; block_size as usize];
        program_data = stream.try_into().expect("msg");

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
            block_size as usize,
        );
    }
}

pub struct TRBLOCK {
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub trigger_comment: LINK,
    pub trigger_events_number: UINT16,
    pub events: Vec<Event>,
}

impl TRBLOCK {
    fn read(stream: &[u8], little_endian: bool) -> (TRBLOCK, usize) {
        let mut position = 0;

        let block_type = stream.try_into().expect("msg");
        let block_size = utils::read_u16(&stream[position..], little_endian, &mut position);
        let trigger_comment = utils::read_u32(&stream[position..], little_endian, &mut position);
        let trigger_events_number =
            utils::read_u16(&stream[position..], little_endian, &mut position);
        let events =
            TRBLOCK::read_events(&stream[position..], little_endian, trigger_events_number);

        return (
            TRBLOCK {
                block_type,
                block_size,
                trigger_comment,
                trigger_events_number,
                events,
            },
            position + block_size as usize,
        );
    }

    fn read_events(stream: &[u8], little_endian: bool, no_events: u16) -> Vec<Event> {
        let mut events = Vec::new();
        let mut position = 0;

        for _i in 0..no_events - 1 {
            let (event, pos) = Event::read(&stream[position..], little_endian);
            events.push(event);
            position = pos;
        }

        return events;
    }
}

pub struct Event {
    pub trigger_time: REAL,
    pub pre_trigger_time: REAL,
    pub post_trigger_time: REAL,
}

impl Event {
    fn read(stream: &[u8], little_endian: bool) -> (Event, usize) {
        let mut position = 0;
        let trigger_time = utils::read_f64(&stream[position..], little_endian, &mut position);
        let pre_trigger_time = utils::read_f64(&stream[position..], little_endian, &mut position);
        let post_trigger_time = utils::read_f64(&stream[position..], little_endian, &mut position);
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

pub struct SRBLOCK {
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub next: LINK,
    pub data_block: LINK,
    pub samples_reduced_number: UINT32,
    pub time_interval_length: REAL,
}

impl SRBLOCK {
    fn read(stream: &[u8], little_endian: bool) -> (SRBLOCK, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream[position..].try_into().expect("msg");
        position += block_type.len();
        let block_size = utils::read_u16(&stream[position..], little_endian, &mut position);
        let next = utils::read_u32(&stream[position..], little_endian, &mut position);
        let data_block = utils::read_u32(&stream[position..], little_endian, &mut position);
        let samples_reduced_number =
            utils::read_u32(&stream[position..], little_endian, &mut position);
        let time_interval_length =
            utils::read_f64(&stream[position..], little_endian, &mut position);

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

pub struct DGBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    next: LINK,
    first: LINK,
    trigger_block: LINK,
    data_block: LINK,
    group_number: UINT16,
    id_number: UINT16,
    reserved: UINT32,
}

impl DGBLOCK {
    // Read the data stream in to a DGBLOCK type, return position reached
    fn read(stream: &[u8], little_endian: bool) -> (DGBLOCK, usize) {
        let mut position = 0;

        // Read block type to confirm
        let block_type: [u8; 2] = stream.try_into().expect("msg");
        position += block_type.len();
        let block_size = utils::read_u16(&stream[position..], little_endian, &mut position);
        let next = utils::read_u32(&stream[position..], little_endian, &mut &mut position);
        let first = utils::read_u32(&stream[position..], little_endian, &mut position);
        let trigger_block = utils::read_u32(&stream[position..], little_endian, &mut position);
        let data_block = utils::read_u32(&stream[position..], little_endian, &mut position);
        let group_number = utils::read_u16(&stream[position..], little_endian, &mut position);
        let id_number = utils::read_u16(&stream[position..], little_endian, &mut position);
        let reserved = utils::read_u32(&stream[position..], little_endian, &mut position);

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
            position,
        );
    }
}

pub struct CGBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    next: LINK,
    first: LINK,
    comment: LINK,
    record_id: UINT16,
    channel_number: UINT16,
    record_size: UINT16,
    record_number: UINT32,
    first_sample_reduction_block: LINK,
}

impl CGBLOCK {
    fn read(stream: &[u8], little_endian: bool) -> (CGBLOCK, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream.try_into().expect("msg");
        let block_size = utils::read_u16(&stream[position..], little_endian, &mut position);
        let next = utils::read_u32(&stream[position..], little_endian, &mut position);
        let first = utils::read_u32(&stream[position..], little_endian, &mut position);
        let comment = utils::read_u32(&stream[position..], little_endian, &mut position);
        let record_id = utils::read_u16(&stream[position..], little_endian, &mut &mut position);
        let channel_number =
            utils::read_u16(&stream[position..], little_endian, &mut &mut position);
        let record_size = utils::read_u16(&stream[position..], little_endian, &mut &mut position);
        let record_number = utils::read_u32(&stream[position..], little_endian, &mut position);
        let first_sample_reduction_block =
            utils::read_u32(&stream[position..], little_endian, &mut position);

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
            position,
        );
    }
}

pub struct CNBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    next: LINK,
    conversion_formula: LINK,
    source_ext: LINK,
    comment: LINK,
    channel_type: UINT16,
    short_name: [CHAR; 32],
    desc: [CHAR; 128],
    start_offset: UINT16,
    bit_number: UINT16,
    data_type: UINT16,
    value_range_valid: BOOL,
    signal_min: REAL,
    signal_max: REAL,
    sample_rate: REAL,
    long_name: LINK,
    display_name: LINK,
    addition_byte_offset: UINT16,
}

impl CNBLOCK {
    fn read(stream: &[u8], little_endian: bool) -> (CNBLOCK, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream.try_into().expect("msg");
        position += block_type.len();
        let block_size = utils::read_u16(&stream[position..], little_endian, &mut &mut position);
        let next = utils::read_u32(&stream[position..], little_endian, &mut position);
        let conversion_formula = utils::read_u32(&stream[position..], little_endian, &mut position);
        let source_ext = utils::read_u32(&stream[position..], little_endian, &mut position);
        let comment = utils::read_u32(&stream[position..], little_endian, &mut position);
        let channel_type = utils::read_u16(&stream[position..], little_endian, &mut position);
        let short_name: [u8; 32] = stream[position..].try_into().expect("msg");
        position += short_name.len();
        let desc: [u8; 128] = stream[position..].try_into().expect("msg");
        position += desc.len();
        let start_offset = utils::read_u16(&stream[position..], little_endian, &mut position);
        let bit_number = utils::read_u16(&stream[position..], little_endian, &mut position);
        let data_type = utils::read_u16(&stream[position..], little_endian, &mut position);
        let value_range_valid = utils::read_u16(&stream[position..], little_endian, &mut position);
        let signal_min = utils::read_f64(&stream[position..], little_endian, &mut position);
        let signal_max = utils::read_f64(&stream[position..], little_endian, &mut position);
        let sample_rate = utils::read_f64(&stream[position..], little_endian, &mut position);
        let long_name = utils::read_u32(&stream[position..], little_endian, &mut position);
        let display_name = utils::read_u32(&stream[position..], little_endian, &mut position);
        let addition_byte_offset =
            utils::read_u16(&stream[position..], little_endian, &mut position);

        return (
            CNBLOCK {
                block_type,
                block_size,
                next,
                conversion_formula,
                source_ext,
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
            position,
        );
    }
}

pub struct CCBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    physical_range_valid: BOOL,
    physical_min: REAL,
    physical_max: REAL,
    unit: [CHAR; 20],
    conversion_type: UINT16,
    size_info: UINT16,
    conversion_data: Conversion_Data,
}

impl CCBLOCK {
    fn read(stream: &[u8], little_endian: bool) -> (CCBLOCK, usize) {
        let mut position = 0;
        let block_type: [CHAR; 2] = stream.try_into().expect("msg");
        position += block_type.len();
        let block_size: UINT16 = utils::read_u16(&stream[position..], little_endian, &mut position);
        let physical_range_valid: BOOL =
            utils::read_u16(&stream[position..], little_endian, &mut position);
        let physical_min: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let physical_max: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let unit: [CHAR; 20] = stream[position..].try_into().expect("msg");
        position += unit.len();
        let conversion_type: UINT16 =
            utils::read_u16(&stream[position..], little_endian, &mut position);
        let size_info: UINT16 = utils::read_u16(&stream[position..], little_endian, &mut position);


		let datatype = 1;


        let (conversion_data, pos) =
            Conversion_Data::read(&stream[position..], little_endian, datatype);
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

// trait ConversionBlock {
//     fn read(stream: &[u8], little_endian: bool) -> (Box<dyn ConversionBlock>, usize);
// }

enum Conversion_Data {
    Parameters,
    Table,
    Text,
}

impl Conversion_Data {
	fn read(data: &[u8], little_endian: bool, datatype: u8 ) -> (Conversion_Data, usize){
		if datatype == 1 {
			return (Conversion_Data::Parameters, 1)
		} else {
			return (Conversion_Data::Table, 1)
		}
	}
}

// impl Conversion_Data {
//     fn read(stream: &[u8], little_endian: bool) -> (Conversion_Data, usize) {}
// }

enum Parameters {
    ConversionLinear,
    ConversionPoly,
    ConversionExponetial,
    ConversionLog,
    ConversionRational,
}

impl Parameters {
	fn read(data: &[u8], little_endian: bool) -> (Parameters, usize) {
		return (
			Parameters::ConversionLinear, 10
		)
	}
}

pub struct ConversionLinear {
    p1: REAL,
    p2: REAL,
}

impl ConversionLinear {
    fn read(stream: &[u8], little_endian: bool) -> (ConversionLinear, usize) {
        let mut position = 0;
        let p1 = utils::read_f64(stream, little_endian, &mut position);
        let p2 = utils::read_f64(&stream[position..], little_endian, &mut &mut position);

        return (ConversionLinear { p1, p2 }, position);
    }
}

pub struct ConversionPoly {
    p1: REAL,
    p2: REAL,
    p3: REAL,
    p4: REAL,
    p5: REAL,
    p6: REAL,
}

impl ConversionPoly {
    fn read(stream: &[u8], little_endian: bool) -> (ConversionPoly, usize) {
        let mut position = 0;
        let p1: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p2: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p3: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p4: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p5: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p6: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);

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

pub struct ConversionExponetial {
    p1: REAL,
    p2: REAL,
    p3: REAL,
    p4: REAL,
    p5: REAL,
    p6: REAL,
    p7: REAL,
}

impl ConversionExponetial {
    fn read(stream: &[u8], little_endian: bool) -> (ConversionExponetial, usize) {
        let mut position = 0;
        let p1: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p2: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p3: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p4: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p5: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p6: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p7: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);

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

pub struct ConversionLog {
    p1: REAL,
    p2: REAL,
    p3: REAL,
    p4: REAL,
    p5: REAL,
    p6: REAL,
    p7: REAL,
}

impl ConversionLog {
    fn read(stream: &[u8], little_endian: bool) -> (ConversionLog, usize) {
        let mut position = 0;
        let p1: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p2: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p3: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p4: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p5: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p6: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p7: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);

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

pub struct ConversionRational {
    p1: REAL,
    p2: REAL,
    p3: REAL,
    p4: REAL,
    p5: REAL,
    p6: REAL,
}

impl ConversionRational {
    fn read(stream: &[u8], little_endian: bool) -> (ConversionRational, usize) {
        let mut position = 0;
        let p1: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p2: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p3: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p4: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p5: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);
        let p6: REAL = utils::read_f64(&stream[position..], little_endian, &mut position);

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

enum Table {
    ConversionTabular,
}

pub struct ConversionTabular {
    value: Vec<TableEntry>,
}

impl ConversionTabular {
    fn read(stream: &[u8], little_endian: bool) -> (ConversionTabular, usize) {

		let mut position = 0;
		let mut value = Vec::new();
		for _i in 0..1 {
			let (temp, pos) = TableEntry::read(&stream[position..], little_endian);
			position += pos;
			value.push(temp);
		}

		return (
			ConversionTabular{
				value, 
			}, 
			position
		)
	}
}

pub struct TableEntry {
    internal: REAL,
    physical: REAL,
}

impl TableEntry {
    fn read(stream: &[u8], little_endian: bool) -> (TableEntry, usize) {
        let mut position = 0;
        let internal = utils::read_f64(&stream[position..], little_endian, &mut &mut position);
        let physical = utils::read_f64(&stream[position..], little_endian, &mut position);

        return (TableEntry { internal, physical }, position);
    }
}

enum Text {
    Conversion_TextFormula,
    Conversion_TextRangeTable,
}

pub struct Conversion_TextFormula {
    formula: [CHAR; 256],
}

impl Conversion_TextFormula {
    fn read(stream: &[u8], little_endian: bool) -> (Conversion_TextFormula, usize) {
        let mut position = 0;
        let formula: [CHAR; 256] = stream.try_into().expect("msg");
        position += formula.len();

        return (Conversion_TextFormula { formula }, position);
    }
}

struct Conversion_TextTable {
    table: Vec<TextTableEntry>,
}

impl Conversion_TextTable {
    fn read(stream: &[u8], little_endian: bool, number: usize) -> (Conversion_TextTable, usize) {
        let mut position = 0;
        let mut table = Vec::new();
        for _i in 0..number - 1 {
            let (table_entry, pos) = TextTableEntry::read(&stream[position..], little_endian);
            table.push(table_entry);
            position += pos;
        }

        return (Conversion_TextTable { table }, position);
    }
}

pub struct TextTableEntry {
    internal: REAL,
    text: [CHAR; 32],
}

impl TextTableEntry {
    fn read(stream: &[u8], little_endian: bool) -> (TextTableEntry, usize) {
        let mut position = 0;
        let internal = utils::read_f64(stream, little_endian, &mut position);
        let text: [CHAR; 32] = stream[position..].try_into().expect("msg");

        return (TextTableEntry { internal, text }, position);
    }
}

pub struct Conversion_TextRangeTable {
    undef1: REAL,
    undef2: REAL,
    txblock: LINK,
    entry: Vec<TextRange>,
}

impl Conversion_TextRangeTable {
    fn read(stream: &[u8], little_endian: bool) -> (Conversion_TextRangeTable, usize) {
        let mut position = 0;
        let undef1 = utils::read_f64(&stream[position..], little_endian, &mut position);
        let undef2 = utils::read_f64(&stream[position..], little_endian, &mut position);
        let txblock = utils::read_u32(&stream[position..], little_endian, &mut &mut position);
		let entry = Vec::new();


		return (
			Conversion_TextRangeTable{
				undef1,
				undef2,
				txblock,
				entry,
			}, 
			position
		)
    }
}

pub struct TextRange {
    lower: REAL,
    upper: REAL,
    txblock: LINK,
}

impl TextRange {
    fn read(stream: &[u8], little_endian: bool) -> (TextRange, usize) {
        let mut position = 0;
        let lower = utils::read_f64(&stream[position..], little_endian, &mut &mut position);
        let upper = utils::read_f64(&stream[position..], little_endian, &mut &mut position);
        let txblock = utils::read_u32(&stream[position..], little_endian, &mut position);

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

pub struct DateStruct {
    ms: UINT16,
    min: BYTE,
    hour: BYTE,
    day: BYTE,
    month: BYTE,
    year: BYTE,
}

impl DateStruct {
    fn read(stream: &[u8], little_endian: bool) -> (DateStruct, usize) {
        let mut position = 0;
        let ms = utils::read_u16(&stream[position..], little_endian, &mut position);
        let min = utils::read_u8(&stream[position..], little_endian, &mut position);
        let hour = utils::read_u8(&stream[position..], little_endian, &mut position);
        let day = utils::read_u8(&stream[position..], little_endian, &mut position);
        let month = utils::read_u8(&stream[position..], little_endian, &mut position);
        let year = utils::read_u8(&stream[position..], little_endian, &mut position);

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

pub struct Time_Struct {
    ms: UINT32,
    days: BYTE,
}

impl Time_Struct {
    fn read(stream: &[u8], little_endian: bool) -> (Time_Struct, usize) {
        let mut position = 0;
        let ms = utils::read_u32(&stream[position..], little_endian, &mut position);
        let days = utils::read_u8(&stream[position..], little_endian, &mut position);

        return (Time_Struct { ms, days }, position);
    }
}

pub struct CDBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    dependency_type: UINT16,
    signal_number: UINT16,
    groups: Vec<Signal>,
    dims: Vec<UINT16>,
}

impl CDBLOCK {
    fn read(stream: &[u8], little_endian: bool) -> (CDBLOCK, usize) {
        let mut position = 0;
        let block_type: [CHAR; 2] = stream.try_into().expect("msg");
        let block_size: UINT16 = utils::read_u16(&stream[position..], little_endian, &mut position);
        let dependency_type: UINT16 =
            utils::read_u16(&stream[position..], little_endian, &mut position);
        let signal_number: UINT16 =
            utils::read_u16(&stream[position..], little_endian, &mut position);

        let mut groups = Vec::new();

        for _i in 0..signal_number - 1 {
            let (temp, pos) = Signal::read(&stream[position..], little_endian);
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
            dims.push(utils::read_u16(
                &stream[position..],
                little_endian,
                &mut position,
            ))
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

pub struct Signal {
    data_group: LINK,
    channel_group: LINK,
    channel: LINK,
}

impl Signal {
    fn read(stream: &[u8], little_endian: bool) -> (Signal, usize) {
        let mut position = 0;
        let data_group = utils::read_u32(&stream[position..], little_endian, &mut position);
        let channel_group = utils::read_u32(&stream[position..], little_endian, &mut position);
        let channel = utils::read_u32(&stream[position..], little_endian, &mut position);

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

pub struct CEBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    extension_type: UINT16,
    additional: Supplement,
}

impl CEBLOCK {
    fn read(stream: &[u8], little_endian: bool) -> (CEBLOCK, usize) {
        let mut position = 0;
        let block_type: [CHAR; 2] = stream.try_into().expect("msg");
        position += block_type.len();
        let block_size: UINT16 = utils::read_u16(&stream[position..], little_endian, &mut position);
        let extension_type: UINT16 =
            utils::read_u16(&stream[position..], little_endian, &mut position);
        let additional: Supplement = Supplement::read(&stream[position..], little_endian);

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

enum Supplement {
    DIMBlock,
    VectorBlock,
}

impl Supplement {
	fn read(stream: &[u8], little_endian: bool) -> Supplement {
		return Supplement::DIMBlock;
	}
}

pub struct DIMBlock {
    module_number: UINT16,
    address: UINT32,
    desc: [CHAR; 80],
    ecu_id: [CHAR; 32],
}

impl DIMBlock {
    fn read(stream: &[u8], little_endian: bool) -> (DIMBlock, usize) {
        let mut position = 0;
        let module_number: UINT16 =
            utils::read_u16(&stream[position..], little_endian, &mut position);
        let address: UINT32 = utils::read_u32(&stream[position..], little_endian, &mut position);
        let desc: [CHAR; 80] = stream[position..].try_into().expect("msg");
        position += desc.len();
        let ecu_id: [CHAR; 32] = stream[position..].try_into().expect("msg");
        position += ecu_id.len();

        return (
            DIMBlock {
                module_number,
                address,
                desc,
                ecu_id,
            },
            position,
        );
    }
}

pub struct VectorBlock {
    can_id: UINT32,
    can_channel: UINT32,
    message_name: [CHAR; 36],
    sender_name: [CHAR; 36],
}

impl VectorBlock {
    fn read(stream: &[u8], little_endian: bool) -> (VectorBlock, usize) {
        let mut position = 0;
        let can_id: UINT32 = utils::read_u32(&stream[position..], little_endian, &mut position);
        let can_channel: UINT32 =
            utils::read_u32(&stream[position..], little_endian, &mut position);
        let message_name: [CHAR; 36] = stream[position..].try_into().expect("msg");
        position += message_name.len();
        let sender_name: [CHAR; 36] = stream[position..].try_into().expect("msg");
        position += sender_name.len();

        return (
            VectorBlock {
                can_id,
                can_channel,
                message_name,
                sender_name,
            },
            position,
        );
    }
}
