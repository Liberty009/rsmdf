use std::mem;

use crate::record;

#[derive(PartialEq, Eq)]
pub enum ChannelHierarchyType {
    Group,
    Function,
    Structure,
    MapList,
    FunctionInput,
    FunctionOutput,
    FunctionLocal,
    FunctionCalDef,
    FunctionCalRef,
}

impl ChannelHierarchyType {
    pub fn new(ch_type: u8) -> Self {
        match ch_type {
            0 => Self::Group,
            1 => Self::Function,
            2 => Self::Structure,
            3 => Self::MapList,
            4 => Self::FunctionInput,
            5 => Self::FunctionOutput,
            6 => Self::FunctionLocal,
            7 => Self::FunctionCalDef,
            8 => Self::FunctionCalRef,
            _ => panic!("Unknown channel type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    Recording,

    RecordingInterrupt,

    AcquistionInterrupt,

    StartRecordingTrigger,

    StopRecordingTrigger,

    Trigger,

    Marker,
}

impl EventType {
    pub fn new(ev_type: u8) -> Self {
        match ev_type {
            0 => Self::Recording,
            1 => Self::RecordingInterrupt,
            2 => Self::AcquistionInterrupt,
            3 => Self::StartRecordingTrigger,
            4 => Self::StopRecordingTrigger,
            5 => Self::Trigger,
            6 => Self::Marker,
            _ => panic!("Error with Event Type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub enum EventSyncType {
    Seconds,

    Radians,

    Meters,

    Index,
}

impl EventSyncType {
    pub fn new(ev_sync: u8) -> Self {
        match ev_sync {
            1 => Self::Seconds,
            2 => Self::Radians,
            3 => Self::Meters,
            4 => Self::Index,
            _ => panic!("Error Event Sync Type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub enum RangeType {
    Point,

    RangeBegin,

    RangeEnd,
}

impl RangeType {
    pub fn new(ev_range: u8) -> Self {
        match ev_range {
            0 => Self::Point,
            1 => Self::RangeBegin,
            2 => Self::RangeEnd,
            _ => panic!("Error Range Type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventCause {
    Other,

    Error,

    Tool,

    Script,

    User,
}

impl EventCause {
    pub fn new(ev_cause: u8) -> Self {
        match ev_cause {
            0 => Self::Other,
            1 => Self::Error,
            2 => Self::Tool,
            3 => Self::Script,
            4 => Self::User,
            _ => panic!("Error Event cause"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    Other,

    Ecu,

    Bus,

    IO,

    Tool,

    User,
}

impl SourceType {
    pub fn new(source: u8) -> Self {
        match source {
            0 => Self::Other,
            1 => Self::Ecu,
            2 => Self::Bus,
            3 => Self::IO,
            4 => Self::Tool,
            5 => Self::User,
            _ => panic!("Error source type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusType {
    None,

    Other,

    Can,

    Lin,

    Most,

    FlexRay,

    KLine,

    Ethernet,

    Usb,
}

impl BusType {
    pub fn new(source: u8) -> Self {
        match source {
            0 => Self::None,
            1 => Self::Other,
            2 => Self::Can,
            3 => Self::Lin,
            4 => Self::Most,
            5 => Self::FlexRay,
            6 => Self::KLine,
            7 => Self::Ethernet,
            8 => Self::Usb,
            _ => panic!("Error bus type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZipType {
    Deflate,

    TransposeDeflate,
}

impl ZipType {
    pub fn new(zip: u8) -> Self {
        match zip {
            0 => Self::Deflate,
            1 => Self::TransposeDeflate,
            _ => panic!("Error zip type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelType {
    FixedLength,
    VariableLength,
    Master,
    VirtualMaster,
    Sync,
    MaxLengthData,
    VirtualData,
}
impl ChannelType {
    pub fn new(channel_type: u8) -> Self {
        match channel_type {
            0 => Self::FixedLength,
            1 => Self::VariableLength,
            2 => Self::Master,
            3 => Self::VirtualMaster,
            4 => Self::Sync,
            5 => Self::MaxLengthData,
            6 => Self::VirtualData,
            _ => panic!("Error: Unknown channel type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncType {
    None,
    Time,
    Angle,
    Distance,
    Index,
}
impl SyncType {
    pub fn new(channel_type: u8) -> Self {
        match channel_type {
            0 => Self::None,
            1 => Self::Time,
            2 => Self::Angle,
            3 => Self::Distance,
            4 => Self::Index,
            _ => panic!("Error: Unknown sync type"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    UnsignedByteLE,
    UnsignedByteBE,
    SignedLE,
    SignedBE,
    FloatLE,
    FloatBE,
    StringLatin,
    StringUTF8,
    StringUTF16LE,
    StringUTF16BE,
    ByteArray,
    MIMESample,
    CANopenData,
    CANopenTime,
}
impl DataType {
    pub fn new(channel_type: u8) -> Self {
        match channel_type {
            0 => Self::UnsignedByteLE,
            1 => Self::UnsignedByteBE,
            2 => Self::SignedLE,
            3 => Self::SignedBE,
            4 => Self::FloatLE,
            5 => Self::FloatBE,
            6 => Self::StringLatin,
            7 => Self::StringUTF8,
            8 => Self::StringUTF16LE,
            9 => Self::StringUTF16BE,
            10 => Self::ByteArray,
            11 => Self::MIMESample,
            12 => Self::CANopenData,
            13 => Self::CANopenTime,
            _ => panic!("Error: Unknown data type"),
        }
    }

    pub fn copy_to_data_type_read(&self) -> record::DataTypeRead {
        let dt = match self {
            Self::UnsignedByteLE => record::DataType::UnsignedInt,
            Self::UnsignedByteBE => record::DataType::UnsignedInt,
            Self::SignedLE => record::DataType::SignedInt,
            Self::SignedBE => record::DataType::SignedInt,
            Self::FloatLE => record::DataType::Float64,
            Self::FloatBE => record::DataType::Float64,
            Self::StringLatin => record::DataType::StringNullTerm,
            Self::StringUTF8 => panic!(""),
            Self::StringUTF16LE => panic!(""),
            Self::StringUTF16BE => panic!(""),
            Self::ByteArray => record::DataType::ByteArray,
            Self::MIMESample => panic!(""),
            Self::CANopenData => panic!(""),
            Self::CANopenTime => panic!(""),
        };

        let end = match self {
            Self::UnsignedByteLE => true,
            Self::UnsignedByteBE => false,
            Self::SignedLE => true,
            Self::SignedBE => false,
            Self::FloatLE => true,
            Self::FloatBE => false,
            Self::StringLatin => false,
            Self::StringUTF8 => panic!(""),
            Self::StringUTF16LE => panic!(""),
            Self::StringUTF16BE => panic!(""),
            Self::ByteArray => false,
            Self::MIMESample => panic!(""),
            Self::CANopenData => panic!(""),
            Self::CANopenTime => panic!(""),
        };

        record::DataTypeRead {
            data_type: dt,
            little_endian: end,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::UnsignedByteLE => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::UnsignedByteBE => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::SignedLE => mem::size_of::<i8>() / mem::size_of::<u8>(),
            Self::SignedBE => mem::size_of::<i8>() / mem::size_of::<u8>(),
            Self::FloatLE => mem::size_of::<f64>() / mem::size_of::<u8>(),
            Self::FloatBE => mem::size_of::<f64>() / mem::size_of::<u8>(),
            Self::StringLatin => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::StringUTF8 => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::StringUTF16LE => mem::size_of::<u16>() / mem::size_of::<u8>(),
            Self::StringUTF16BE => mem::size_of::<u16>() / mem::size_of::<u8>(),
            Self::ByteArray => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::MIMESample => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::CANopenData => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::CANopenTime => mem::size_of::<u8>() / mem::size_of::<u8>(),
            // _ => panic!("")
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CCType {
    Direct,
    Parametic,
    Rational,
    Algebraic,
    ValueTableInterpolate,
    ValueTableNoInterpolate,
    RangeTableValue,
    ValueTableText,
    RangeTableText,
    TextTableValue,
    TextTableText,
}
impl CCType {
    pub fn new(cc_type: u8) -> Self {
        match cc_type {
            0 => Self::Direct,
            1 => Self::Parametic,
            2 => Self::Rational,
            3 => Self::Algebraic,
            4 => Self::ValueTableInterpolate,

            5 => Self::ValueTableNoInterpolate,

            6 => Self::RangeTableValue,
            7 => Self::ValueTableText,
            8 => Self::RangeTableText,
            9 => Self::TextTableValue,
            10 => Self::TextTableText,
            _ => panic!("Error CCtype"),
        }
    }
}
