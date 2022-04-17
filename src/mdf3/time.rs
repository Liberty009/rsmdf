use crate::utils;

#[derive(Debug, Clone, Copy)]
pub struct TimeStruct {
    pub ms: u32,
    pub days: u8,
}

impl TimeStruct {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], little_endian: bool) -> (TimeStruct, usize) {
        let mut position = 0;
        let ms = utils::read(stream, little_endian, &mut position);
        let days = utils::read(stream, little_endian, &mut position);

        (TimeStruct { ms, days }, position)
    }
}
