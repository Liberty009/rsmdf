use crate::utils;

#[derive(Debug, Clone, Copy)]
pub struct DateStruct {
    pub ms: u16,
    pub min: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u8,
}

impl DateStruct {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], little_endian: bool) -> (DateStruct, usize) {
        let mut position = 0;
        let ms = utils::read(stream, little_endian, &mut position);
        let min = utils::read(stream, little_endian, &mut position);
        let hour = utils::read(stream, little_endian, &mut position);
        let day = utils::read(stream, little_endian, &mut position);
        let month = utils::read(stream, little_endian, &mut position);
        let year = utils::read(stream, little_endian, &mut position);

        (
            DateStruct {
                ms,
                min,
                hour,
                day,
                month,
                year,
            },
            position,
        )
    }
}
