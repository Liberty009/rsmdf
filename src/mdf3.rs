use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};

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

struct TXBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    text: Vec<CHAR>,
}

struct PRBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    program_data: Vec<CHAR>,
}

struct TRBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    trigger_comment: LINK,
    trigger_events_number: UINT16,
    events: Vec<Event>,
}

struct Event {
    trigger_time: REAL,
    pre_trigger_time: REAL,
    post_trigger_time: REAL,
}

struct SRBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    next: LINK,
    data_block: LINK,
    samples_reduced_number: UINT32,
    time_interval_length: REAL,
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
    fn new(datastream: &[u8], little_endian: bool) -> (DGBLOCK, usize) {
        // Read block type to confirm
        let block_type = [datastream[0], datastream[1]];
        let block_size = if little_endian {
            LittleEndian::read_u16(&[datastream[2], datastream[3]])
        } else {
            BigEndian::read_u16(&[datastream[2], datastream[3]])
        };
        let next = if little_endian {
            LittleEndian::read_u32(&[datastream[4], datastream[5], datastream[6], datastream[7]])
        } else {
            BigEndian::read_u32(&[datastream[4], datastream[5], datastream[6], datastream[7]])
        };

        let first = if little_endian {
            LittleEndian::read_u32(&[datastream[8], datastream[9], datastream[10], datastream[11]])
        } else {
            BigEndian::read_u32(&[datastream[8], datastream[9], datastream[10], datastream[11]])
        };

        let trigger_block = if little_endian {
            LittleEndian::read_u32(&[
                datastream[12],
                datastream[13],
                datastream[14],
                datastream[15],
            ])
        } else {
            BigEndian::read_u32(&[
                datastream[12],
                datastream[13],
                datastream[14],
                datastream[15],
            ])
        };
        let data_block = if little_endian {
            LittleEndian::read_u32(&[
                datastream[16],
                datastream[17],
                datastream[18],
                datastream[19],
            ])
        } else {
            BigEndian::read_u32(&[
                datastream[16],
                datastream[17],
                datastream[18],
                datastream[19],
            ])
        };

        let group_number = if little_endian {
            LittleEndian::read_u16(&[datastream[20], datastream[21]])
        } else {
            BigEndian::read_u16(&[datastream[20], datastream[21]])
        };

        let id_number = if little_endian {
            LittleEndian::read_u16(&[datastream[22], datastream[23]])
        } else {
            BigEndian::read_u16(&[datastream[22], datastream[23]])
        };

        let reserved = if little_endian {
            LittleEndian::read_u32(&[
                datastream[24],
                datastream[25],
                datastream[26],
                datastream[27],
            ])
        } else {
            BigEndian::read_u32(&[
                datastream[24],
                datastream[25],
                datastream[26],
                datastream[27],
            ])
        };

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
            28,
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

enum Conversion_Data {
    Parameters,
    Table,
    Text,
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

struct Conversion_Poly {
    p1: REAL,
    p2: REAL,
    p3: REAL,
    p4: REAL,
    p5: REAL,
    p6: REAL,
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

struct Conversion_Log {
    p1: REAL,
    p2: REAL,
    p3: REAL,
    p4: REAL,
    p5: REAL,
    p6: REAL,
    p7: REAL,
}

struct Conversion_Rational {
    p1: REAL,
    p2: REAL,
    p3: REAL,
    p4: REAL,
    p5: REAL,
    p6: REAL,
}

enum Table {
    Conversion_Tabular,
}

struct Conversion_Tabular {
    value: Vec<TableEntry>,
}

struct TableEntry {
    internal: REAL,
    physical: REAL,
}

enum Text {
    Conversion_TextFormula,
    Conversion_TextRangeTable,
}

struct Conversion_TextFormula {
    formula: [CHAR; 256],
}

struct Conversion_TextTable {
    table: Vec<(REAL, [CHAR; 32])>,
}

struct Conversion_TextRangeTable {
    undef1: REAL,
    undef2: REAL,
    txblock: LINK,
    entry: Vec<TextRange>,
}

struct TextRange {
    lower: REAL,
    upper: REAL,
    txblock: LINK,
}

struct Date_Struct {
    ms: UINT16,
    min: BYTE,
    hour: BYTE,
    day: BYTE,
    month: BYTE,
    year: BYTE,
}

struct Time_Struct {
    ms: UINT32,
    days: BYTE,
}
struct CDBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    dependency_type: UINT16,
    signal_number: UINT16,
    groups: Vec<Signal>,
    dims: Vec<UINT16>,
}

struct Signal {
    data_group: LINK,
    channel_group: LINK,
    channel: LINK,
}
struct CEBLOCK {
    block_type: [CHAR; 2],
    block_size: UINT16,
    extension_type: UINT16,
    additional: Supplement,
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

struct VectorBlock {
    can_id: UINT32,
    can_channel: UINT32,
    message_name: [CHAR; 36],
    sender_name: [CHAR; 36],
}
