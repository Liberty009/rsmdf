use crate::utils;

#[derive(Debug, Clone, PartialEq)]
pub struct Signals {
    pub data_group: u32,
    pub channel_group: u32,
    pub channel: u32,
}

impl Signals {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
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
