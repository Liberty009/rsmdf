use std::{convert::TryInto, f32::MIN_POSITIVE, intrinsics::exp2f64, mem, ops::Bound, ptr::read, sync::PoisonError};

use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use itertools::Powerset;

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

struct IDBLOCK {
    file_id: [CHAR; 8],
    format_id: [CHAR; 8],
    program_id: [CHAR; 8],
    default_byte_order: UINT16,
    default_float_format: UINT16,
    version_number: UINT16,
    code_page_number: UINT16,
    reserved1: [CHAR; 2],
    reserved2: [CHAR; 30],
}

impl IDBLOCK {
    fn read(stream: &[u8], little_endian: bool) -> (IDBLOCK, usize) {
        let file_id = stream[0..7].try_into().expect("msg");
        let format_id = stream[8..].try_into().expect("msg");
        let program_id = stream[16..].try_into().expect("msg");
        let default_byte_order = if little_endian {
            LittleEndian::read_u16(&stream[24..])
        } else {
            BigEndian::read_u16(&stream[24..])
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
        );
    }
}

struct HDBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    data_group_block: LINK,
    file_comment: LINK,
    program_block: LINK,
    data_group_number: UINT16,
    date: [CHAR; 10],
    time: [CHAR; 8],
    author: [CHAR; 32],
    department: [CHAR; 32],
    project: [CHAR; 32],
    subject: [CHAR; 32],
    timestamp: UINT64,
    utc_time_offset: INT16,
    time_quality: UINT16,
    timer_id: [CHAR; 32],
}

impl HDBLOCK {
    fn read(stream: &[u8], little_endian: bool) -> (HDBLOCK, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream[0..].try_into().expect("");
        position += block_type.len();
        let block_size = read_u16(&stream[position..], little_endian, &mut position);
        let data_group_block = read_u32(&stream[position..], little_endian, &mut position);
        let file_comment = read_u32(&stream[position..], little_endian, &mut position);
        let program_block = read_u32(&stream[position..], little_endian, &mut position);
        let data_group_number = read_u16(&stream[position..], little_endian, &mut position);
        let date: [u8; 10] = stream[position..].try_into().expect("msg");
        position += date.len();
        let time: [u8; 8] = stream[position..].try_into().expect("msg");
        position += time.len();
        let author: [u8; 32] = stream[position..].try_into().expect("msg");
        position += author.len();
        let department: [u8; 32] = stream[position..].try_into().expect("msg");
        position += department.len();
        let project: [u8; 32] = stream[position..].try_into().expect("msg");
        position += project.len();
        let subject: [u8; 32] = stream[position..].try_into().expect("msg");
        position += subject.len();
        let timestamp = read_u64(&stream[position..], little_endian, &mut position);
        let utc_time_offset = read_i16(&stream[position..], little_endian, &mut position);
        let time_quality = read_u16(&stream[position..], little_endian, &mut position);
        let timer_id: [u8; 32] = stream[position..].try_into().expect("msg");
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

struct TXBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    text: Vec<CHAR>,
}

impl TXBLOCK {
    fn read(stream: &[u8], little_endian: bool) -> (TXBLOCK, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream.try_into().expect("");
        position += block_type.len();
        let block_size = read_u16(&stream[position..], little_endian, &mut position);

        let mut text: Vec<u8> = vec![0; block_size as usize];
        text = stream.try_into().expect("msg");
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

struct PRBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    program_data: Vec<CHAR>,
}

impl PRBLOCK {
    fn read(stream: &[u8], little_endian: bool) -> (PRBLOCK, usize) {
        let block_type = stream.try_into().expect("");
        let block_size = if little_endian {
            LittleEndian::read_u16(&stream[2..])
        } else {
            BigEndian::read_u16(&stream[2..])
        };

        let mut program_data = vec![0; block_size as usize];
        program_data = stream.try_into().expect("msg");

        return (
            PRBLOCK {
                block_type,
                block_size,
                program_data,
            },
            20,
        );
    }
}

struct TRBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    trigger_comment: LINK,
    trigger_events_number: UINT16,
    events: Vec<Event>,
}

impl TRBLOCK {
    fn read(stream: &[u8], little_endian: bool) -> (TRBLOCK, usize) {
        let mut position = 0;

        let block_type = stream.try_into().expect("msg");
        let block_size = read_u16(&stream[position..], little_endian, &mut position);
        let trigger_comment = read_u32(&stream[position..], little_endian, &mut position);
        let trigger_events_number = read_u16(&stream[position..], little_endian, &mut position);
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

fn read_u16(stream: &[u8], little_endian: bool, position: &mut usize) -> u16 {
    *position += mem::size_of::<u16>() / mem::size_of::<u8>();
    if little_endian {
        return LittleEndian::read_u16(stream);
    } else {
        return BigEndian::read_u16(stream);
    }
}

fn read_i16(stream: &[u8], little_endian: bool, position: &mut usize) -> i16 {
    *position += mem::size_of::<i16>() / mem::size_of::<u8>();
    if little_endian {
        return LittleEndian::read_i16(stream);
    } else {
        return BigEndian::read_i16(stream);
    }
}

fn read_u32(stream: &[u8], little_endian: bool, position: &mut usize) -> u32 {
    *position += mem::size_of::<u32>() / mem::size_of::<u8>();

    if little_endian {
        return LittleEndian::read_u32(stream);
    } else {
        return BigEndian::read_u32(stream);
    }
}

fn read_u64(stream: &[u8], little_endian: bool, position: &mut usize) -> u64 {
    *position += mem::size_of::<u64>() / mem::size_of::<u8>();

    if little_endian {
        return LittleEndian::read_u64(stream);
    } else {
        return BigEndian::read_u64(stream);
    }
}

fn read_f64(stream: &[u8], little_endian: bool, position: &mut usize) -> f64 {
    *position += mem::size_of::<f64>() / mem::size_of::<u8>();
    if little_endian {
        return LittleEndian::read_f64(stream);
    } else {
        return BigEndian::read_f64(stream);
    }
}

struct Event {
    trigger_time: REAL,
    pre_trigger_time: REAL,
    post_trigger_time: REAL,
}

impl Event {
    fn read(stream: &[u8], little_endian: bool) -> (Event, usize) {
        let mut position = 0;
        let trigger_time = read_f64(&stream[position..], little_endian, &mut position);
        let pre_trigger_time = read_f64(&stream[position..], little_endian, &mut position);
        let post_trigger_time = read_f64(&stream[position..], little_endian, &mut position);
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

struct SRBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    next: LINK,
    data_block: LINK,
    samples_reduced_number: UINT32,
    time_interval_length: REAL,
}

impl SRBLOCK {
    fn read(stream: &[u8], little_endian: bool) -> (SRBLOCK, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream[position..].try_into().expect("msg");
        position += block_type.len();
        let block_size = read_u16(&stream[position..], little_endian, &mut position);
        let next = read_u32(&stream[position..], little_endian, &mut position);
        let data_block = read_u32(&stream[position..], little_endian, &mut position);
        let samples_reduced_number = read_u32(&stream[position..], little_endian, &mut position);
        let time_interval_length = read_f64(&stream[position..], little_endian, &mut position);

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

struct DGBLOCK {
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
        let block_size = read_u16(&stream[position..], little_endian, &mut position);
        let next = read_u32(&stream[position..], little_endian, &mut &mut position);
        let first = read_u32(&stream[position..], little_endian, &mut position);
        let trigger_block = read_u32(&stream[position..], little_endian, &mut position);
        let data_block = read_u32(&stream[position..], little_endian, &mut position);
        let group_number = read_u16(&stream[position..], little_endian, &mut position);
        let id_number = read_u16(&stream[position..], little_endian, &mut position);
        let reserved = read_u32(&stream[position..], little_endian, &mut position);

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

struct CGBLOCK {
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
        let block_size = read_u16(&stream[position..], little_endian, &mut position);
        let next = read_u32(&stream[position..], little_endian, &mut position);
        let first = read_u32(&stream[position..], little_endian, &mut position);
        let comment = read_u32(&stream[position..], little_endian, &mut position);
        let record_id = read_u16(&stream[position..], little_endian, &mut &mut position);
        let channel_number = read_u16(&stream[position..], little_endian, &mut &mut position);
        let record_size = read_u16(&stream[position..], little_endian, &mut &mut position);
        let record_number = read_u32(&stream[position..], little_endian, &mut position);
        let first_sample_reduction_block =
            read_u32(&stream[position..], little_endian, &mut position);

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

struct CNBLOCK {
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
        let block_size = read_u16(&stream[position..], little_endian, &mut &mut position);
        let next = read_u32(&stream[position..], little_endian, &mut position);
        let conversion_formula = read_u32(&stream[position..], little_endian, &mut position);
        let source_ext = read_u32(&stream[position..], little_endian, &mut position);
        let comment = read_u32(&stream[position..], little_endian, &mut position);
        let channel_type = read_u16(&stream[position..], little_endian, &mut position);
        let short_name: [u8; 32] = stream[position..].try_into().expect("msg");
        position += short_name.len();
        let desc: [u8; 128] = stream[position..].try_into().expect("msg");
        position += desc.len();
        let start_offset = read_u16(&stream[position..], little_endian, &mut position);
        let bit_number = read_u16(&stream[position..], little_endian, &mut position);
        let data_type = read_u16(&stream[position..], little_endian, &mut position);
        let value_range_valid = read_u16(&stream[position..], little_endian, &mut position);
        let signal_min = read_f64(&stream[position..], little_endian, &mut position);
        let signal_max = read_f64(&stream[position..], little_endian, &mut position);
        let sample_rate = read_f64(&stream[position..], little_endian, &mut position);
        let long_name = read_u32(&stream[position..], little_endian, &mut position);
        let display_name = read_u32(&stream[position..], little_endian, &mut position);
        let addition_byte_offset = read_u16(&stream[position..], little_endian, &mut position);

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

struct CCBLOCK {
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
        let block_size: UINT16 = read_u16(&stream[position..], little_endian, &mut position);
        let physical_range_valid: BOOL =
            read_u16(&stream[position..], little_endian, &mut position);
        let physical_min: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let physical_max: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let unit: [CHAR; 20] = stream[position..].try_into().expect("msg");
        position += unit.len();
        let conversion_type: UINT16 = read_u16(&stream[position..], little_endian, &mut position);
        let size_info: UINT16 = read_u16(&stream[position..], little_endian, &mut position);
        let (conversion_data, pos): Conversion_Data =
            Conversion_Data::read(&stream[position..], little_endian);
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

trait ConversionBlock {
    fn read(stream: &[u8], little_endian: bool) -> (Box<dyn ConversionBlock>, usize);
}

enum Conversion_Data {
    Parameters,
    Table,
    Text,
}

impl ConversionBlock for Conversion_Data {
    fn read(stream: &[u8], little_endian: bool) -> (Conversion_Data, usize) {}
}

enum Parameters {
    Conversion_Linear,
    Conversion_Poly,
    Conversion_Exponetial,
    Conversion_Log,
    Conversion_Rational,
}

struct Conversion_Linear {
    p1: REAL,
    p2: REAL,
}

impl ConversionBlock for Conversion_Linear {
    fn read(stream: &[u8], little_endian: bool) -> (Conversion_Linear, usize) {
        let mut position = 0;
        let p1 = read_f64(stream, little_endian, &mut position);
        let p2 = read_f64(&stream[position..], little_endian, &mut &mut position);

        return (Conversion_Linear { p1, p2 }, position);
    }
}

struct Conversion_Poly {
    p1: REAL,
    p2: REAL,
    p3: REAL,
    p4: REAL,
    p5: REAL,
    p6: REAL,
}

impl Conversion_Poly {
    fn read(stream: &[u8], little_endian: bool) -> (Conversion_Poly, usize) {
        let mut position = 0;
        let p1: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p2: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p3: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p4: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p5: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p6: REAL = read_f64(&stream[position..], little_endian, &mut position);

        return (
            Conversion_Poly {
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

struct Conversion_Exponetial {
    p1: REAL,
    p2: REAL,
    p3: REAL,
    p4: REAL,
    p5: REAL,
    p6: REAL,
    p7: REAL,
}

impl Conversion_Exponetial {
    fn read(stream: &[u8], little_endian: bool) -> (Conversion_Exponetial, usize) {
        let mut position = 0;
        let p1: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p2: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p3: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p4: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p5: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p6: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p7: REAL = read_f64(&stream[position..], little_endian, &mut position);

        return (
            Conversion_Exponetial {
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

struct Conversion_Log {
    p1: REAL,
    p2: REAL,
    p3: REAL,
    p4: REAL,
    p5: REAL,
    p6: REAL,
    p7: REAL,
}

impl Conversion_Log {
    fn read(stream: &[u8], little_endian: bool) -> (Conversion_Log, usize) {
        let mut position = 0;
        let p1: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p2: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p3: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p4: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p5: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p6: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p7: REAL = read_f64(&stream[position..], little_endian, &mut position);

        return (
            Conversion_Log {
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

struct Conversion_Rational {
    p1: REAL,
    p2: REAL,
    p3: REAL,
    p4: REAL,
    p5: REAL,
    p6: REAL,
}

impl Conversion_Rational {
    fn read(stream: &[u8], little_endian: bool) -> (Conversion_Rational, usize) {
        let mut position = 0;
        let p1: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p2: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p3: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p4: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p5: REAL = read_f64(&stream[position..], little_endian, &mut position);
        let p6: REAL = read_f64(&stream[position..], little_endian, &mut position);

        return (
            Conversion_Rational {
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
    Conversion_Tabular,
}

struct Conversion_Tabular {
    value: Vec<TableEntry>,
}

impl Conversion_Tabular {
    fn read(stream: &[u8], little_endian: bool) -> (Conversion_Tabular, usize) {}
}

struct TableEntry {
    internal: REAL,
    physical: REAL,
}

impl TableEntry {
    fn read(stream: &[u8], little_endian: bool) -> (TableEntry, usize) {
        let mut position = 0;
        let internal = read_f64(&stream[position..], little_endian, &mut &mut position);
        let physical = read_f64(&stream[position..], little_endian, &mut position);

        return (TableEntry { internal, physical }, position);
    }
}

enum Text {
    Conversion_TextFormula,
    Conversion_TextRangeTable,
}

struct Conversion_TextFormula {
    formula: [CHAR; 256],
}

impl Conversion_TextFormula {
    fn read(stream: &[u8], little_endian: bool) -> (Conversion_TextFormula, usize) {
        let mut position = 0;
        let formula = stream.try_into().expect("msg");
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

struct TextTableEntry {
    internal: REAL,
    text: [CHAR; 32],
}

impl TextTableEntry {
    fn read(stream: &[u8], little_endian: bool) -> (TextTableEntry, usize) {
        let mut position = 0;
        let internal = read_f64(stream, little_endian, &mut position);
        let text = &stream[position..].try_into().expect("msg");

        return (TextTableEntry { internal, text }, position);
    }
}

struct Conversion_TextRangeTable {
    undef1: REAL,
    undef2: REAL,
    txblock: LINK,
    entry: Vec<TextRange>,
}

impl Conversion_TextRangeTable {
    fn read(&stream: &[u8], little_endian: bool) -> (Conversion_TextRangeTable, usize) {
        let mut position = 0;
        let undef1 = read_f64(&stream[position..], little_endian, &mut position);
        let undef2 = read_f64(&stream[position..], little_endian, &mut position);
        let txblock = read_u32(&stream[position..], little_endian, &mut &mut position);
    }
}

struct TextRange {
    lower: REAL,
    upper: REAL,
    txblock: LINK,
}

impl TextRange {
    fn read(&stream: &[u8], little_endian: bool) -> (TextRange, usize) {
        let mut position = 0;
        let lower = read_f64(&stream[position..], little_endian, &mut &mut position);
        let upper = read_f64(&stream[position..], little_endian, &mut &mut position);
        let txblock = read_u32(&stream[position..], little_endian, &mut position);

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

struct Date_Struct {
    ms: UINT16,
    min: BYTE,
    hour: BYTE,
    day: BYTE,
    month: BYTE,
    year: BYTE,
}

fn read_u8(stream: &[u8], little_endian: bool, position: &mut usize) -> u8 {
    *position += 1;
    return stream[0];
}

impl Date_Struct {
    fn read(stream: &[u8], little_endian: bool) -> (Date_Struct, usize) {
        let mut position = 0;
        let ms = read_u16(&stream[position..], little_endian, &mut position);
        let min = read_u8(&stream[position..], little_endian, &mut position);
        let hour = read_u8(&stream[position..], little_endian, &mut position);
        let day = read_u8(&stream[position..], little_endian, &mut position);
        let month = read_u8(&stream[position..], little_endian, &mut position);
        let year = read_u8(&stream[position..], little_endian, &mut position);

        return (
            Date_Struct {
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

struct Time_Struct {
    ms: UINT32,
    days: BYTE,
}

impl Time_Struct {
    fn read(stream: &[u8], little_endian: bool) -> (Time_Struct, usize) {
        let mut position = 0;
        let ms = read_u32(&stream[position..], little_endian, &mut position);
        let days = read_u8(&stream[position..], little_endian, &mut position);

        return (Time_Struct { ms, days }, position);
    }
}

struct CDBLOCK {
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
        let block_size: UINT16 = read_u16(&stream[position..], little_endian, &mut position);
        let dependency_type: UINT16 = read_u16(&stream[position..], little_endian, &mut position);
        let signal_number: UINT16 = read_u16(&stream[position..], little_endian, &mut position);

        let mut groups = Vec::new();

        for _i in 0..signal_number - 1 {
            let (temp, pos) = Signal::read(&stream[position..], little_endian, signal_number);
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
            dims.push(read_u16(&stream[position..], little_endian, &mut position))
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

struct Signal {
    data_group: LINK,
    channel_group: LINK,
    channel: LINK,
}

impl Signal {
    fn read(stream: &[u8], little_endian: bool, number: usize) -> (Signal, usize) {
        let mut position = 0;
        let data_group = read_u32(&stream[position..], little_endian, &mut position);
        let channel_group = read_u32(&stream[position..], little_endian, &mut position);
        let channel = read_u32(&stream[position..], little_endian, &mut position);

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

struct CEBLOCK {
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
		let block_size: UINT16 = read_u16(&stream[position..], little_endian, &mut position);
		let extension_type: UINT16 = read_u16(&stream[position..], little_endian, &mut position);
		let additional: Supplement = Supplement::read(&stream[position..], little_endian);

		return (
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

enum Supplement {
    DIMBlock,
    VectorBlock,
}

struct DIMBlock {
    module_number: UINT16,
    address: UINT32,
    desc: [CHAR; 80],
    ecu_id: [CHAR; 32],
}

impl DIMBlock {
	fn read(stream: &[u8], little_endian: bool) -> (DIMBlock, usize) {
		let mut position = 0;
		let module_number: UINT16 = read_u16(&stream[position..], little_endian, &mut position);
		let address: UINT32 = read_u32(&stream[position..], little_endian, &mut position);
		let desc: [CHAR; 80] = &stream[position..].try_into().expect("msg");
		position += desc.len();
		let ecu_id: [CHAR; 32] = &stream[position..].try_into().expect("msg");
		position += ecu_id.len();

		return (
			DIMBlock{
				module_number,
				address,
				desc,
				ecu_id,
			}, 
			position
		)
	}
}

struct VectorBlock {
    can_id: UINT32,
    can_channel: UINT32,
    message_name: [CHAR; 36],
    sender_name: [CHAR; 36],
}

impl  VectorBlock {
	fn read(stream: &[u8], little_endian: bool) -> (VectorBlock, usize) {
		let mut position = 0;
		let can_id: UINT32 = read_u32(&stream[position..], little_endian, &mut position);
		let can_channel: UINT32 = read_u32(&stream[position..], little_endian, &mut position);
		let message_name: [CHAR; 36] = &stream[position..].try_into().expect("msg");
		position += message_name.len();
		let sender_name: [CHAR; 36] = &stream[position..].try_into().expect("msg");
		position += sender_name.len();

		return (
			VectorBlock{
				can_id,
				can_channel,
				message_name,
				sender_name,
			}, 
			position,
		)
	}
}
