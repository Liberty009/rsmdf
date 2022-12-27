use crate::{record::DataTypeRead, utils};

use super::{
    channel_type::ChannelType,
    mdf3_block::{LinkedBlock, Mdf3Block},
    tx_block::Txblock,
};

#[derive(Debug, Clone, Copy)]
pub struct Cnblock {
    block_type: [u8; 2],
    block_size: u16,
    next: u32,
    conversion_formula: u32,
    source_ext: u32,
    dependency: u32,
    comment: u32,
    channel_type: u16,
    short_name: [u8; 32],
    desc: [u8; 128],
    start_offset: u16,
    bit_number: u16,
    data_type: DataTypeRead,
    value_range_valid: u16,
    signal_min: f64,
    signal_max: f64,
    sample_rate: f64,
    long_name: u32,
    display_name: u32,
    addition_byte_offset: u16,
}

impl LinkedBlock for Cnblock {
    fn next(&self, stream: &[u8], little_endian: bool) -> Option<Self>
    where
        Self: std::marker::Sized,
    {
        if self.next == 0 {
            None
        } else {
            let (_pos, block) = Self::read(stream, self.next as usize, little_endian);
            Some(block)
        }
    }

    fn list(&self, stream: &[u8], little_endian: bool) -> Vec<Self>
    where
        Self: std::marker::Sized,
    {
        let mut all = Vec::new();

        let next = self.next(stream, little_endian);

        all.push(*self);
        match next {
            None => {}
            Some(block) => all.append(&mut block.list(stream, little_endian)),
        }

        all
    }
}

impl Mdf3Block for Cnblock {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let block_type: [u8; 2] = utils::read(stream, little_endian, &mut pos);
        if !utils::eq(&block_type, "CN".as_bytes()) {
            panic!("CNBLOCK not found.");
        }

        let block_size = utils::read(stream, little_endian, &mut pos);
        let next = utils::read(stream, little_endian, &mut pos);
        let conversion_formula = utils::read(stream, little_endian, &mut pos);
        let source_ext = utils::read(stream, little_endian, &mut pos);
        let dependency = utils::read(stream, little_endian, &mut pos);
        let comment = utils::read(stream, little_endian, &mut pos);
        let channel_type = utils::read(stream, little_endian, &mut pos);

        let short_name: [u8; 32] = utils::read(stream, little_endian, &mut pos);

        let desc: [u8; 128] = utils::read(stream, little_endian, &mut pos);

        let start_offset = utils::read(stream, little_endian, &mut pos);
        let bit_number = utils::read(stream, little_endian, &mut pos);

        let datatype: u16 = utils::read(stream, little_endian, &mut pos);
        let data_type = DataTypeRead::new(datatype, little_endian);

        let value_range_valid = utils::read(stream, little_endian, &mut pos);
        let signal_min = utils::read(stream, little_endian, &mut pos);
        let signal_max = utils::read(stream, little_endian, &mut pos);
        let sample_rate = utils::read(stream, little_endian, &mut pos);
        let long_name = utils::read(stream, little_endian, &mut pos);
        let display_name = utils::read(stream, little_endian, &mut pos);
        let addition_byte_offset = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Cnblock {
                block_type,
                block_size,
                next,
                conversion_formula,
                source_ext,
                dependency,
                comment,
                channel_type,
                short_name,
                desc,
                start_offset,
                bit_number,
                data_type,
                value_range_valid,
                signal_min,
                signal_max,
                sample_rate,
                long_name,
                display_name,
                addition_byte_offset,
            },
        )
    }

    fn write(&self, _start_position: usize, little_endian: bool) -> Vec<u8> {
        let mut array = Vec::new();

        array.append(&mut self.block_type.to_vec());
        array.append(&mut utils::write(self.block_size, little_endian));
        array.append(&mut utils::write(self.next, little_endian));
        array.append(&mut utils::write(self.conversion_formula, little_endian));
        array.append(&mut utils::write(self.source_ext, little_endian));
        array.append(&mut utils::write(self.dependency, little_endian));
        array.append(&mut utils::write(self.comment, little_endian));
        array.append(&mut utils::write(self.channel_type, little_endian));
        array.append(&mut self.short_name.to_vec());
        array.append(&mut self.desc.to_vec());
        array.append(&mut utils::write(self.start_offset, little_endian));
        array.append(&mut utils::write(self.bit_number, little_endian));
        array.append(&mut self.data_type.write(little_endian));
        array.append(&mut utils::write(self.value_range_valid, little_endian));
        array.append(&mut utils::write(self.signal_min, little_endian));
        array.append(&mut utils::write(self.signal_max, little_endian));
        array.append(&mut utils::write(self.sample_rate, little_endian));
        array.append(&mut utils::write(self.long_name, little_endian));
        array.append(&mut utils::write(self.display_name, little_endian));
        array.append(&mut utils::write(self.addition_byte_offset, little_endian));

        array
    }
}

impl Cnblock {
    pub fn channel_type(&self) -> ChannelType {
        ChannelType::new(self.channel_type)
    }
    pub fn byte_offset(&self) -> usize {
        self.start_offset as usize / 8
    }

    pub fn data_type_len(&self) -> usize {
        self.data_type.len()
    }

    pub fn data_type(&self) -> DataTypeRead {
        self.data_type
    }

    pub fn name(self, stream: &[u8], little_endian: bool) -> String {
        let mut name = "".to_string();

        if self.channel_type == 1 {
            name = "time".to_string();
        } else if self.long_name != 0 {
            let (_pos, tx) = Txblock::read(stream, self.long_name as usize, little_endian);
            tx.name();
            // name = match std::str::from_utf8(&tx.text) {
            //     Ok(v) => v.to_string(),
            //     Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            // };
        }

        name
    }
}

#[cfg(test)]
mod tests {
    use crate::utils;

    use super::*;

    static CN_DATA: [u8; 274] = [
        0x43, 0x4E, 0xE4, 0x00, 0xA6, 0xE3, 0x10, 0x00, 0x80, 0xE0, 0x10, 0x00, 0xAE, 0xE0, 0x10,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x74, 0x69, 0x6D, 0x65,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x03, 0x00, 0x01, 0x00, 0x04,
        0x19, 0x60, 0x9C, 0xAE, 0xDD, 0xBC, 0x3F, 0x52, 0xE8, 0x62, 0xFA, 0x56, 0xD3, 0x28, 0x40,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x54, 0x58, 0x2B, 0x00, 0x41, 0x53, 0x41, 0x4D, 0x2E, 0x4D, 0x2E, 0x53,
        0x43, 0x41, 0x4C, 0x41, 0x52, 0x2E, 0x53, 0x42, 0x59, 0x54, 0x45, 0x2E, 0x49, 0x44, 0x45,
        0x4E, 0x54, 0x49, 0x43, 0x41, 0x4C, 0x2E, 0x44, 0x49, 0x53, 0x43, 0x52, 0x45, 0x54, 0x45,
        0x00, 0x54, 0x58, 0xBB,
    ];

    #[test]
    fn read() {
        let (_pos, cn_block) = Cnblock::read(&CN_DATA, 0, true);

        //assert_eq!(position, 228);
        assert_eq!(cn_block.block_size, 228);
        assert_eq!(cn_block.next, 1106854);
        assert_eq!(cn_block.conversion_formula, 1106048);
        assert_eq!(cn_block.source_ext, 1106094);
        assert_eq!(cn_block.dependency, 0);
        assert_eq!(cn_block.comment, 0);
        assert_eq!(cn_block.channel_type, 1);
        assert!(utils::eq(
            &cn_block.short_name,
            &[
                0x74, 0x69, 0x6D, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ]
        ));

        assert!(utils::eq(
            &cn_block.desc,
            &[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00
            ]
        ));
        assert_eq!(cn_block.start_offset, 0);
        assert_eq!(cn_block.bit_number, 64);
        // assert_eq!(cn_block.data_type, mdf3::DataType::Float64);
        assert_eq!(cn_block.value_range_valid, 1);

        assert!(
            (cn_block.signal_min
                - utils::read::<f64>(
                    &[0x04, 0x19, 0x60, 0x9C, 0xAE, 0xDD, 0xBC, 0x3F,],
                    true,
                    &mut 0_usize
                ))
            .abs()
                < 0.1
        );
        assert!(
            (cn_block.signal_max
                - utils::read::<f64>(
                    &[0x52, 0xE8, 0x62, 0xFA, 0x56, 0xD3, 0x28, 0x40,],
                    true,
                    &mut 0_usize
                ))
            .abs()
                < 0.1
        );
        assert!((cn_block.sample_rate - 0.0).abs() < 0.1);

        assert_eq!(cn_block.display_name, 0);
        assert_eq!(cn_block.addition_byte_offset, 0);
    }

    #[test]
    fn write() {
        let (_pos, cn_block) = Cnblock::read(&CN_DATA, 0, true);

        let write_array = cn_block.write(0, true);

        assert!(utils::eq(&CN_DATA, &write_array));
    }
}
