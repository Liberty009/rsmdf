use crate::utils;


#[derive(Debug, Clone, Copy)]
pub struct Srblock {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub next: u32,
    pub data_block: u32,
    pub samples_reduced_number: u32,
    pub time_interval_length: f64,
}

impl Srblock {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], little_endian: bool) -> (Srblock, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream.try_into().expect("msg");
        position += block_type.len();
        let block_size = utils::read(stream, little_endian, &mut position);
        let next = utils::read(stream, little_endian, &mut position);
        let data_block = utils::read(stream, little_endian, &mut position);
        let samples_reduced_number = utils::read(stream, little_endian, &mut position);
        let time_interval_length = utils::read(stream, little_endian, &mut position);

        (
            Srblock {
                block_type,
                block_size,
                next,
                data_block,
                samples_reduced_number,
                time_interval_length,
            },
            position,
        )
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn read() {}

    #[test]
    fn write() {}
}
