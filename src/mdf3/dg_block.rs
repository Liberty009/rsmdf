use crate::utils;

use super::{cg_block::Cgblock, mdf3_block::Mdf3Block};

#[derive(Debug, Clone, Copy)]
pub struct Dgblock {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub next: u32,
    pub first: u32,
    pub trigger_block: u32,
    pub data_block: u32,
    pub group_number: u16,
    pub id_number: u16,
    pub reserved: u32,
}

impl Mdf3Block for Dgblock {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;

        // Read block type to confirm
        let block_type: [u8; 2] = utils::read(stream, little_endian, &mut pos);
        if !utils::eq(&block_type, "DG".as_bytes()) {
            panic!(
                "DGBLOCK not found. Found: {}, {}",
                block_type[0], block_type[1]
            );
        }

        let block_size = utils::read(stream, little_endian, &mut pos);
        let next = utils::read(stream, little_endian, &mut pos);
        let first = utils::read(stream, little_endian, &mut pos);
        let trigger_block = utils::read(stream, little_endian, &mut pos);
        let data_block = utils::read(stream, little_endian, &mut pos);
        let group_number = utils::read(stream, little_endian, &mut pos);
        let id_number = utils::read(stream, little_endian, &mut pos);
        let reserved = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Dgblock {
                block_type,
                block_size,
                next,
                first,
                trigger_block,
                data_block,
                group_number,
                id_number,
                reserved,
            },
        )
    }
}

impl Dgblock {
    pub fn write() {}

    pub fn read_all(stream: &[u8], little_endian: bool, position: usize) -> Vec<Self> {
        let mut all = Vec::new();
        let mut next_dg = position;

        while next_dg != 0 {
            let (_pos, dg_block) = Dgblock::read(stream, next_dg, little_endian);
            next_dg = dg_block.next as usize;
            all.push(dg_block);
        }

        all
    }

    pub fn read_channel_groups(self, stream: &[u8], little_endian: bool) -> Vec<Cgblock> {
        let mut channel_grps = Vec::new();
        let mut next = self.first as usize;
        while next != 0 {
            let (_pos, cg_block) = Cgblock::read(stream, next, little_endian);
            next = cg_block.next as usize;
            channel_grps.push(cg_block);
        }
        channel_grps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read() {
        let dg_data = [
            0x44, 0x47, 0x1C, 0x00, 0xF4, 0xDF, 0x10, 0x00, 0x99, 0xE4, 0x10, 0x00, 0x2B, 0xE5,
            0x10, 0x00, 0xDC, 0x03, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let position = 0;
        let (position, dg_block) = Dgblock::read(&dg_data, position, true);

        assert_eq!(position, 28);
        assert_eq!(dg_block.next, 1105908);
        assert_eq!(dg_block.first, 1107097);
        assert_eq!(dg_block.trigger_block, 1107243);
        assert_eq!(dg_block.data_block, 988);
        assert_eq!(dg_block.group_number, 1);
        assert_eq!(dg_block.id_number, 0);
        assert_eq!(dg_block.reserved, 0);
    }

    #[test]
    fn write() {}
}
