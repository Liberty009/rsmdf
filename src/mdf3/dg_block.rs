use crate::utils;

use super::{
    cg_block::Cgblock,
    mdf3_block::{LinkedBlock, Mdf3Block},
};

#[derive(Debug, Clone, Copy)]
pub struct Dgblock {
    #[allow(dead_code)]
    block_type: [u8; 2],
    #[allow(dead_code)]
    block_size: u16,
    #[allow(dead_code)]
    next: u32,
    #[allow(dead_code)]
    first: u32,
    #[allow(dead_code)]
    trigger_block: u32,
    #[allow(dead_code)]
    data_block: u32,
    #[allow(dead_code)]
    group_number: u16,
    #[allow(dead_code)]
    id_number: u16,
    #[allow(dead_code)]
    reserved: u32,
}

impl LinkedBlock for Dgblock {
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

    fn write(&self, _start_position: usize, little_endian: bool) -> Vec<u8> {
        let mut array = Vec::new();

        array.append(&mut self.block_type.to_vec());
        array.append(&mut utils::write(self.block_size, little_endian));
        array.append(&mut utils::write(self.next, little_endian));
        array.append(&mut utils::write(self.first, little_endian));
        array.append(&mut utils::write(self.trigger_block, little_endian));
        array.append(&mut utils::write(self.data_block, little_endian));
        array.append(&mut utils::write(self.group_number, little_endian));
        array.append(&mut utils::write(self.id_number, little_endian));
        array.append(&mut utils::write(self.reserved, little_endian));

        array
    }
}

impl Dgblock {
    #[allow(dead_code)]
    pub fn data_location(&self) -> usize {
        self.data_block as usize
    }

    pub fn first_channel_group(&self, stream: &[u8], little_endian: bool) -> Cgblock {
        if self.first == 0 {
            panic!("Error");
        }

        let (_pos, cg) = Cgblock::read(stream, self.first as usize, little_endian);

        cg
    }

    pub fn read_data(&self, stream: &[u8], _little_endian: bool, channel: &Cgblock) -> Vec<u8> {
        let data_length = self.data_length(channel);
        let data_block = self.data_block as usize;

        stream[data_block..data_block + data_length].to_vec()
    }

    #[allow(dead_code)]
    pub fn write() {
        todo!()
    }

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
        let first_group = self.first_channel_group(stream, little_endian);
        first_group.list(stream, little_endian)
    }

    fn data_length(self, channel: &Cgblock) -> usize {
        let record_number = channel.record_number();
        let record_size = channel.record_size();
        record_number * record_size
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
