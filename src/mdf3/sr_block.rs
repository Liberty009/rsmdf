use crate::utils;

use super::mdf3_block::{LinkedBlock, Mdf3Block};

#[derive(Debug, Clone, Copy)]
pub struct Srblock {
    block_type: [u8; 2],

    block_size: u16,

    next: u32,

    data_block: u32,

    samples_reduced_number: u32,

    time_interval_length: f64,
}

impl LinkedBlock for Srblock {
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

    fn write(&self, _start_position: usize, little_endian: bool) -> Vec<u8> {
        let mut array = Vec::new();

        array.append(&mut self.block_type.to_vec());
        array.append(&mut utils::write(self.block_size, little_endian));
        array.append(&mut utils::write(self.next, little_endian));
        array.append(&mut utils::write(self.data_block, little_endian));
        array.append(&mut utils::write(
            self.samples_reduced_number,
            little_endian,
        ));
        array.append(&mut utils::write(self.time_interval_length, little_endian));

        array
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn read() {}

    #[test]
    fn write() {}
}
