use crate::utils;
use std::{convert::TryInto};


use byteorder::{BigEndian, ByteOrder, LittleEndian};

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

pub fn read(file: &[u8]) -> (IDBLOCK, bool, usize) {
	let (id_block, position, little_endian) = IDBLOCK::read(file);
	return (id_block, little_endian, position);
}

pub fn read_head(file: &[u8], little_endian: bool) -> (HDBLOCK, usize) {
	let (hdblock, position) = HDBLOCK::read(file, little_endian);
	return (hdblock, position)
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

impl IDBLOCK {
    pub fn read(stream: &[u8]) -> (IDBLOCK, usize, bool) {
		let mut position = 0;
        let file_id: [u8;8] = stream[position..position + 8].try_into().expect("msg");

		if !utils::eq(&file_id[..], &[0x4D, 0x44, 0x46, 0x20, 0x20, 0x20, 0x20, 0x20,]) {
			panic!("Error: Incorrect file type");
		}

		position += file_id.len();

        let format_id: [u8; 8] = stream[position..position+8].try_into().expect("msg");
		position += format_id.len();

        let program_id: [u8; 8] = stream[position..position+8].try_into().expect("msg");
		position += program_id.len();

        let default_byte_order = LittleEndian::read_u16(&stream[position..]);
		position += 2;

		let little_endian = if default_byte_order == 0 {
			true
		} else {
			false
		};

        let default_float_format = if little_endian {
            LittleEndian::read_u16(&stream[position..])
        } else {
            BigEndian::read_u16(&stream[position..])
        };
		position += 2;

        let version_number = if little_endian {
            LittleEndian::read_u16(&stream[position..])
        } else {
            BigEndian::read_u16(&stream[position..])
        };
		position += 2;

        let code_page_number = if little_endian {
            LittleEndian::read_u16(&stream[position..])
        } else {
            BigEndian::read_u16(&stream[position..])
        };
		position += 2;

        let reserved1: [u8; 2] = [stream[position], stream[position + 1]];
		position += 2;
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

		if !utils::eq(&block_type, &['H' as u8, 'D' as u8]) {
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

        let mut text: Vec<u8> = stream[position..position+block_size as usize-5].try_into().expect("msg");
		
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
		if !utils::eq(&block_type, &['P' as u8, 'R' as u8,]) {
			panic!("PR Block not found");
		}
        let block_size = if little_endian {
            LittleEndian::read_u16(&stream[2..])
        } else {
            BigEndian::read_u16(&stream[2..])
        };

        //let mut program_data = vec![0; block_size as usize];
        let mut program_data: Vec<u8> = stream.try_into().expect("msg");

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
    pub fn read(stream: &[u8], little_endian: bool) -> (TRBLOCK, usize) {
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
    pub fn read(stream: &[u8], little_endian: bool) -> (SRBLOCK, usize) {
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
    pub fn read(stream: &[u8], little_endian: bool) -> (DGBLOCK, usize) {
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
    pub fn read(stream: &[u8], little_endian: bool) -> (CGBLOCK, usize) {
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
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub next: LINK,
    pub conversion_formula: LINK,
    pub source_ext: LINK,
    pub comment: LINK,
    pub channel_type: UINT16,
    pub short_name: [CHAR; 32],
    pub desc: [CHAR; 128],
    pub start_offset: UINT16,
    pub bit_number: UINT16,
    pub data_type: UINT16,
    pub value_range_valid: BOOL,
    pub signal_min: REAL,
    pub signal_max: REAL,
    pub sample_rate: REAL,
    pub long_name: LINK,
    pub display_name: LINK,
    pub addition_byte_offset: UINT16,
}

impl CNBLOCK {
    pub fn read(stream: &[u8], little_endian: bool) -> (CNBLOCK, usize) {
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
            ConversionData::read(&stream[position..], little_endian, datatype);
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

pub enum ConversionData {
    Parameters,
    Table,
    Text,
}

impl ConversionData {
	pub fn read(_data: &[u8], _little_endian: bool, datatype: u8 ) -> (ConversionData, usize){
		if datatype == 1 {
			return (ConversionData::Parameters, 1)
		} else {
			return (ConversionData::Table, 1)
		}
	}
}

pub enum Parameters {
    ConversionLinear,
    ConversionPoly,
    ConversionExponetial,
    ConversionLog,
    ConversionRational,
}

impl Parameters {
	pub fn read(_data: &[u8], _little_endian: bool) -> (Parameters, usize) {
		return (
			Parameters::ConversionLinear, 10
		)
	}
}

pub struct ConversionLinear {
    pub p1: REAL,
    pub p2: REAL,
}

impl ConversionLinear {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionLinear, usize) {
        let mut position = 0;
        let p1 = utils::read_f64(stream, little_endian, &mut position);
        let p2 = utils::read_f64(&stream[position..], little_endian, &mut &mut position);

        return (ConversionLinear { p1, p2 }, position);
    }
}

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

pub enum Table {
    ConversionTabular,
}

pub struct ConversionTabular {
    pub value: Vec<TableEntry>,
}

impl ConversionTabular {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionTabular, usize) {

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
    pub internal: REAL,
    pub physical: REAL,
}

impl TableEntry {
    pub fn read(stream: &[u8], little_endian: bool) -> (TableEntry, usize) {
        let mut position = 0;
        let internal = utils::read_f64(&stream[position..], little_endian, &mut &mut position);
        let physical = utils::read_f64(&stream[position..], little_endian, &mut position);

        return (TableEntry { internal, physical }, position);
    }
}

pub enum Text {
    ConversionTextFormula,
    ConversionTextRangeTable,
}

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

pub struct ConversionTextTable {
    pub table: Vec<TextTableEntry>,
}

impl ConversionTextTable {
    pub fn read(stream: &[u8], little_endian: bool, number: usize) -> (ConversionTextTable, usize) {
        let mut position = 0;
        let mut table = Vec::new();
        for _i in 0..number - 1 {
            let (table_entry, pos) = TextTableEntry::read(&stream[position..], little_endian);
            table.push(table_entry);
            position += pos;
        }

        return (ConversionTextTable { table }, position);
    }
}

pub struct TextTableEntry {
    pub internal: REAL,
    pub text: [CHAR; 32],
}

impl TextTableEntry {
    pub fn read(stream: &[u8], little_endian: bool) -> (TextTableEntry, usize) {
        let mut position = 0;
        let internal = utils::read_f64(stream, little_endian, &mut position);
        let text: [CHAR; 32] = stream[position..].try_into().expect("msg");

        return (TextTableEntry { internal, text }, position);
    }
}

pub struct ConversionTextRangeTable {
    pub undef1: REAL,
    pub undef2: REAL,
    pub txblock: LINK,
    pub entry: Vec<TextRange>,
}

impl ConversionTextRangeTable {
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionTextRangeTable, usize) {
        let mut position = 0;
        let undef1 = utils::read_f64(&stream[position..], little_endian, &mut position);
        let undef2 = utils::read_f64(&stream[position..], little_endian, &mut position);
        let txblock = utils::read_u32(&stream[position..], little_endian, &mut &mut position);
		let entry = Vec::new();


		return (
			ConversionTextRangeTable{
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
    pub lower: REAL,
    pub upper: REAL,
    pub txblock: LINK,
}

impl TextRange {
    pub fn read(stream: &[u8], little_endian: bool) -> (TextRange, usize) {
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

pub struct TimeStruct {
    pub ms: UINT32,
    pub days: BYTE,
}

impl TimeStruct {
    pub fn read(stream: &[u8], little_endian: bool) -> (TimeStruct, usize) {
        let mut position = 0;
        let ms = utils::read_u32(&stream[position..], little_endian, &mut position);
        let days = utils::read_u8(&stream[position..], little_endian, &mut position);

        return (TimeStruct { ms, days }, position);
    }
}

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
    pub data_group: LINK,
    pub channel_group: LINK,
    pub channel: LINK,
}

impl Signal {
    pub fn read(stream: &[u8], little_endian: bool) -> (Signal, usize) {
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
    pub block_type: [CHAR; 2],
    pub block_size: UINT16,
    pub extension_type: UINT16,
    pub additional: Supplement,
}

impl CEBLOCK {
    pub fn read(stream: &[u8], little_endian: bool) -> (CEBLOCK, usize) {
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

pub enum Supplement {
    DIMBlock,
    VectorBlock,
}

impl Supplement {
	pub fn read(_stream: &[u8], _little_endian: bool) -> Supplement {
		return Supplement::DIMBlock;
	}
}

pub struct DIMBlock {
    pub module_number: UINT16,
    pub address: UINT32,
    pub desc: [CHAR; 80],
    pub ecu_id: [CHAR; 32],
}

impl DIMBlock {
    pub fn read(stream: &[u8], little_endian: bool) -> (DIMBlock, usize) {
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
    pub can_id: UINT32,
    pub can_channel: UINT32,
    pub message_name: [CHAR; 36],
    pub sender_name: [CHAR; 36],
}

impl VectorBlock {
    pub fn read(stream: &[u8], little_endian: bool) -> (VectorBlock, usize) {
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
