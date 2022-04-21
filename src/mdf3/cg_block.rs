use crate::utils;

use super::{
    cn_block::Cnblock,
    mdf3_block::{LinkedBlock, Mdf3Block},
    tx_block::Txblock,
};

#[derive(Debug, Clone, Copy)]
pub struct Cgblock {
    #[allow(dead_code)]
    block_type: [u8; 2],
    #[allow(dead_code)]
    block_size: u16,
    #[allow(dead_code)]
    next: u32,
    #[allow(dead_code)]
    first: u32,
    #[allow(dead_code)]
    comment: u32,
    #[allow(dead_code)]
    record_id: u16,
    #[allow(dead_code)]
    number_of_channels: u16,
    #[allow(dead_code)]
    record_size: u16,
    #[allow(dead_code)]
    record_number: u32,
    #[allow(dead_code)]
    first_sample_reduction_block: u32,
}

impl LinkedBlock for Cgblock {
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

impl Mdf3Block for Cgblock {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let block_type: [u8; 2] = utils::read(stream, little_endian, &mut pos);
        // stream[pos..pos + 2].try_into().expect("msg");

        if !utils::eq(&block_type, "CG".as_bytes()) {
            panic!(
                "CGBLOCK not found. Found: {}, {}",
                block_type[0] as char, block_type[1] as char
            );
        }

        let block_size = utils::read(stream, little_endian, &mut pos);
        let next = utils::read(stream, little_endian, &mut pos);
        let first = utils::read(stream, little_endian, &mut pos);
        let comment = utils::read(stream, little_endian, &mut pos);
        let record_id = utils::read(stream, little_endian, &mut pos);
        let channel_number = utils::read(stream, little_endian, &mut pos);
        let record_size = utils::read(stream, little_endian, &mut pos);
        let record_number = utils::read(stream, little_endian, &mut pos);
        let first_sample_reduction_block = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Cgblock {
                block_type,
                block_size,
                next,
                first,
                comment,
                record_id,
                number_of_channels: channel_number,
                record_size,
                record_number,
                first_sample_reduction_block,
            },
        )
    }
}

impl Cgblock {
    #[allow(dead_code)]
    pub fn data_length(&self) -> usize {
        self.record_number as usize * self.record_size as usize
    }

    pub fn record_number(&self) -> usize {
        self.record_number as usize
    }

    pub fn record_size(&self) -> usize {
        self.record_size as usize
    }

    pub fn first_channel(&self, stream: &[u8], little_endian: bool) -> Cnblock {
        if self.first == 0 {
            panic!("Error");
        }

        let (_pos, cn) = Cnblock::read(stream, self.first as usize, little_endian);
        cn
    }

    pub fn comment(&self, stream: &[u8], little_endian: bool) -> String {
        let (_pos, tx) = Txblock::read(stream, self.comment as usize, little_endian);
        tx.name()
    }
    #[allow(dead_code)]
    pub fn write() {}
    pub fn channels(self, stream: &[u8], little_endian: bool) -> Vec<Cnblock> {
        let first_channel = self.first_channel(stream, little_endian);
        first_channel.list(stream, little_endian)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read() {
        let cg_data = [
            0x43, 0x47, 0x1E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2E, 0xE1, 0x10, 0x00, 0x8A, 0xE4,
            0x10, 0x00, 0x01, 0x00, 0x02, 0x00, 0x09, 0x00, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        let (position, cg_block) = Cgblock::read(&cg_data, 0, true);

        assert_eq!(position, 30);

        assert_eq!(cg_block.next, 0);
        assert_eq!(cg_block.first, 1106222);
        assert_eq!(cg_block.comment, 1107082);
        assert_eq!(cg_block.record_id, 1);
        assert_eq!(cg_block.number_of_channels, 2);
        assert_eq!(cg_block.record_size, 9);
        assert_eq!(cg_block.record_number, 124);
        assert_eq!(cg_block.first_sample_reduction_block, 0);
    }

    #[test]
    fn write() {}
}
