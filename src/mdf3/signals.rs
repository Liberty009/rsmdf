use crate::utils;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signals {
    pub data_group: u32,
    pub channel_group: u32,
    pub channel: u32,
}

impl Signals {
    pub fn write(&self, little_endian: bool) -> Vec<u8> {
        let mut array = Vec::new();
        array.append(&mut utils::write(self.data_group, little_endian));
        array.append(&mut utils::write(self.channel_group, little_endian));
        array.append(&mut utils::write(self.channel, little_endian));

        array
    }

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
