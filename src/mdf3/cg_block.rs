use crate::utils;

use super::cn_block::Cnblock;


#[derive(Debug, Clone, Copy)]
pub struct Cgblock {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub next: u32,
    pub first: u32,
    pub comment: u32,
    pub record_id: u16,
    pub channel_number: u16,
    pub record_size: u16,
    pub record_number: u32,
    pub first_sample_reduction_block: u32,
}

impl Cgblock {
    pub fn write() {}
    pub fn read(stream: &[u8], little_endian: bool, position: usize) -> (Cgblock, usize) {
        let mut pos = position;
        let block_type: [u8; 2] = stream[pos..pos + 2].try_into().expect("msg");

        if !utils::eq(&block_type, &[b'C', b'G']) {
            panic!(
                "CGBLOCK not found. Found: {}, {}",
                block_type[0] as char, block_type[1] as char
            );
        }

        pos += block_type.len();

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
            Cgblock {
                block_type,
                block_size,
                next,
                first,
                comment,
                record_id,
                channel_number,
                record_size,
                record_number,
                first_sample_reduction_block,
            },
            pos,
        )
    }
    pub fn channels(self, stream: &[u8], little_endian: bool) -> Vec<Cnblock> {
        //let (group, _) = Self::read(stream, little_endian, position);
        let mut ch = Vec::new();
        let mut next_cn = self.first as usize;
        while next_cn != 0 {
            let (cn_block, _position) = Cnblock::read(stream, little_endian, next_cn);
            next_cn = cn_block.next as usize;

            ch.push(cn_block);
        }

        ch
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

        let (cg_block, position) = Cgblock::read(&cg_data, true, 0);

        assert_eq!(position, 30);

        assert_eq!(cg_block.next, 0);
        assert_eq!(cg_block.first, 1106222);
        assert_eq!(cg_block.comment, 1107082);
        assert_eq!(cg_block.record_id, 1);
        assert_eq!(cg_block.channel_number, 2);
        assert_eq!(cg_block.record_size, 9);
        assert_eq!(cg_block.record_number, 124);
        assert_eq!(cg_block.first_sample_reduction_block, 0);
    }

    #[test]
    fn write() {}
}