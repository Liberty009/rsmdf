#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cdblock {
    block_type: [u8; 2],
    block_size: u16,
    dependency_type: u16,
    signal_number: u16,
    groups: Vec<Signals>,
    dims: Vec<u16>,
}

impl Mdf3Block for Cdblock {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let block_type: [u8; 2] = utils::read(stream, little_endian, &mut pos);

        assert!(utils::eq(&block_type, "CD".as_bytes()));

        let block_size = utils::read(stream, little_endian, &mut pos);
        let dependency_type = utils::read(stream, little_endian, &mut pos);
        let signal_number = utils::read(stream, little_endian, &mut pos);

        let mut groups = Vec::with_capacity(signal_number as usize);

        for _i in 0..signal_number - 1 {
            let (temp, pos_sig) = Signals::read(stream, little_endian);
            groups.push(temp);
            pos += pos_sig;
        }

        let mut dims = Vec::new();

        let no_dependencies = if dependency_type < 255 {
            dependency_type
        } else {
            dependency_type - 255
        };
        for _i in 0..no_dependencies - 1 {
            dims.push(utils::read(stream, little_endian, &mut pos))
        }

        (
            pos,
            Cdblock {
                block_type,
                block_size,
                dependency_type,
                signal_number,
                groups,
                dims,
            },
        )
    }

    fn write(&self, _start_position: usize, little_endian: bool) -> Vec<u8> {
        let mut array = self.block_type.to_vec();
        array.append(&mut utils::write(self.block_size, little_endian));
        array.append(&mut utils::write(self.dependency_type, little_endian));
        array.append(&mut utils::write(self.signal_number, little_endian));

        for group in &self.groups {
            array.append(&mut group.write(little_endian));
        }

        for dim in &self.dims {
            array.append(&mut utils::write(*dim, little_endian));
        }

        array
    }
}

impl Cdblock {
    #[allow(dead_code)]
    pub fn write() {}
}

#[cfg(test)]
mod tests {
    use crate::mdf3::cd_block;

    use super::*;

    static CD_DATA: [u8; 113] = [
        0x43, 0x44, 0x01, 0xA8, 0x0C, 0x70, 0xD4, 0x01, 0x90, 0x0B, 0x7E, 0xAF, 0x7C, 0x11, 0xF4,
        0x3F, 0x44, 0x7C, 0xE0, 0x9C, 0x03, 0x00, 0x00, 0x18, 0x7C, 0xE0, 0x44, 0x70, 0x00, 0x00,
        0xD4, 0x43, 0x0C, 0xD4, 0x38, 0x03, 0x00, 0x00, 0x39, 0x00, 0x18, 0x7C, 0x00, 0x00, 0x00,
        0x60, 0x43, 0x44, 0x01, 0xA8, 0x0C, 0x70, 0xD4, 0x01, 0x90, 0xC7, 0xF2, 0x42, 0x95, 0x15,
        0xF4, 0x3F, 0x44, 0x7C, 0xE0, 0x9C, 0x03, 0x00, 0x00, 0x18, 0x7C, 0xE0, 0x44, 0x70, 0x00,
        0x00, 0xD4, 0x43, 0x0C, 0xD4, 0x38, 0x03, 0x00, 0x00, 0x39, 0x00, 0x18, 0x7C, 0x00, 0x00,
        0x00, 0x60, 0x43, 0x44, 0x01, 0xA8, 0x0C, 0x70, 0xD4, 0x01, 0x90, 0x84, 0x67, 0xD6, 0xAD,
        0x19, 0xF4, 0x3F, 0x44, 0x7C, 0xE0, 0x9C, 0x03,
    ];

    #[test]
    fn read() {
        // let (position, cd_block) = Cdblock::read(&CD_DATA, 0, true);

        // assert_eq!(position, 113);
    }

    #[test]
    fn write() {}
}

use crate::{mdf3::signals::Signals, utils};

use super::mdf3_block::Mdf3Block;
