use crate::utils;

use super::mdf3_block::Mdf3Block;

#[derive(Debug, Clone, Copy)]
pub struct Srblock {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub next: u32,
    pub data_block: u32,
    pub samples_reduced_number: u32,
    pub time_interval_length: f64,
}

impl Mdf3Block for Srblock {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let block_type: [u8; 2] = utils::read(stream, little_endian, &mut pos);

        if !utils::eq(&block_type, "SR".as_bytes()) {
            panic!("Expected SR block but found: {:?}", block_type);
        }

        let block_size = utils::read(stream, little_endian, &mut pos);
        let next = utils::read(stream, little_endian, &mut pos);
        let data_block = utils::read(stream, little_endian, &mut pos);
        let samples_reduced_number = utils::read(stream, little_endian, &mut pos);
        let time_interval_length = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Srblock {
                block_type,
                block_size,
                next,
                data_block,
                samples_reduced_number,
                time_interval_length,
            },
        )
    }
}

impl Srblock {
    #[allow(dead_code)]
    pub fn write() {}
}

#[cfg(test)]
mod tests {

    #[test]
    fn read() {}

    #[test]
    fn write() {}
}
