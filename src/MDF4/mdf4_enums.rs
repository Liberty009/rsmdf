use std::mem;

#[allow(dead_code)]
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
    #[allow(dead_code)]
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

#[derive(Debug, Clone)]

pub enum EventType {
    #[allow(dead_code)]
    Recording,
    #[allow(dead_code)]
    RecordingInterrupt,
    #[allow(dead_code)]
    AcquistionInterrupt,
    #[allow(dead_code)]
    StartRecordingTrigger,
    #[allow(dead_code)]
    StopRecordingTrigger,
    #[allow(dead_code)]
    Trigger,
    #[allow(dead_code)]
    Marker,
}

impl EventType {
    #[allow(dead_code)]
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

#[derive(Debug, Clone)]

pub enum EventSyncType {
    #[allow(dead_code)]
    Seconds,
    #[allow(dead_code)]
    Radians,
    #[allow(dead_code)]
    Meters,
    #[allow(dead_code)]
    Index,
}

impl EventSyncType {
    #[allow(dead_code)]
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

#[derive(Debug, Clone)]

pub enum RangeType {
    #[allow(dead_code)]
    Point,
    #[allow(dead_code)]
    RangeBegin,
    #[allow(dead_code)]
    RangeEnd,
}

impl RangeType {
    #[allow(dead_code)]
    pub fn new(ev_range: u8) -> Self {
        match ev_range {
            0 => Self::Point,
            1 => Self::RangeBegin,
            2 => Self::RangeEnd,
            _ => panic!("Error Range Type"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EventCause {
    #[allow(dead_code)]
    Other,
    #[allow(dead_code)]
    Error,
    #[allow(dead_code)]
    Tool,
    #[allow(dead_code)]
    Script,
    #[allow(dead_code)]
    User,
}

impl EventCause {
    #[allow(dead_code)]
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

#[derive(Debug, Clone)]
pub enum SourceType {
    #[allow(dead_code)]
    Other,
    #[allow(dead_code)]
    Ecu,
    #[allow(dead_code)]
    Bus,
    #[allow(dead_code)]
    IO,
    #[allow(dead_code)]
    Tool,
    #[allow(dead_code)]
    User,
}

impl SourceType {
    #[allow(dead_code)]
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

#[derive(Debug, Clone)]
pub enum BusType {
    #[allow(dead_code)]
    None,
    #[allow(dead_code)]
    Other,
    #[allow(dead_code)]
    Can,
    #[allow(dead_code)]
    Lin,
    #[allow(dead_code)]
    Most,
    #[allow(dead_code)]
    FlexRay,
    #[allow(dead_code)]
    KLine,
    #[allow(dead_code)]
    Ethernet,
    #[allow(dead_code)]
    Usb,
}

impl BusType {
    #[allow(dead_code)]
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

#[derive(Debug, Clone)]
pub enum ZipType {
    #[allow(dead_code)]
    Deflate,
    #[allow(dead_code)]
    TransposeDeflate,
}

impl ZipType {
    #[allow(dead_code)]
    pub fn new(zip: u8) -> Self {
        match zip {
            0 => Self::Deflate,
            1 => Self::TransposeDeflate,
            _ => panic!("Error zip type"),
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
    #[allow(dead_code)]
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

#[derive(Debug, Clone)]
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
