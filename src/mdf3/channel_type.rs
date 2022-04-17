pub enum ChannelType{
    Data, 
    Time
}

impl ChannelType {
    pub fn new(channel_type: u16) -> Self {
        match channel_type{
            0 => Self::Data, 
            1 => Self::Time, 
            _ => panic!("Error: Unknown channel type")
        }
    }

    pub fn is_time(&self) -> bool {
        match self {
            Self::Data => false, 
            Self::Time => true
        }
    }
}