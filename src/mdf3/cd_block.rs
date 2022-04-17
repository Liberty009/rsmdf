#[derive(Debug, Clone, PartialEq)]
pub struct Cdblock {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub dependency_type: u16,
    pub signal_number: u16,
    pub groups: Vec<Signals>,
    pub dims: Vec<u16>,
}

impl Cdblock {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], little_endian: bool) -> (Cdblock, usize) {
        let mut position = 0;
        let block_type: [u8; 2] = stream[position..position + 2].try_into().expect("msg");

        assert!(utils::eq(&block_type, &[b'C', b'D']));

        position += block_type.len();
        let block_size: u16 = utils::read(stream, little_endian, &mut position);
        let dependency_type: u16 = utils::read(stream, little_endian, &mut position);
        let signal_number: u16 = utils::read(stream, little_endian, &mut position);

        let mut groups = Vec::with_capacity(signal_number as usize);

        for _i in 0..signal_number - 1 {
            let (temp, pos) = Signals::read(stream, little_endian);
            groups.push(temp);
            position += pos;
        }

        let mut dims = Vec::new();

        let no_dependencies = if dependency_type < 255 {
            dependency_type
        } else {
            dependency_type - 255
        };
        for _i in 0..no_dependencies - 1 {
            dims.push(utils::read(stream, little_endian, &mut position))
        }

        (
            Cdblock {
                block_type,
                block_size,
                dependency_type,
                signal_number,
                groups,
                dims,
            },
            position,
        )
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn read() {
//         let cd_data = [
//             0x43, 0x44, 0x01, 0xA8, 0x0C, 0x70, 0xD4, 0x01, 0x90, 0x0B, 0x7E, 0xAF, 0x7C, 0x11,
//             0xF4, 0x3F, 0x44, 0x7C, 0xE0, 0x9C, 0x03, 0x00, 0x00, 0x18, 0x7C, 0xE0, 0x44, 0x70,
//             0x00, 0x00, 0xD4, 0x43, 0x0C, 0xD4, 0x38, 0x03, 0x00, 0x00, 0x39, 0x00, 0x18, 0x7C,
//             0x00, 0x00, 0x00, 0x60, 0x43, 0x44, 0x01, 0xA8, 0x0C, 0x70, 0xD4, 0x01, 0x90, 0xC7,
//             0xF2, 0x42, 0x95, 0x15, 0xF4, 0x3F, 0x44, 0x7C, 0xE0, 0x9C, 0x03, 0x00, 0x00, 0x18,
//             0x7C, 0xE0, 0x44, 0x70, 0x00, 0x00, 0xD4, 0x43, 0x0C, 0xD4, 0x38, 0x03, 0x00, 0x00,
//             0x39, 0x00, 0x18, 0x7C, 0x00, 0x00, 0x00, 0x60, 0x43, 0x44, 0x01, 0xA8, 0x0C, 0x70,
//             0xD4, 0x01, 0x90, 0x84, 0x67, 0xD6, 0xAD, 0x19, 0xF4, 0x3F, 0x44, 0x7C, 0xE0, 0x9C,
//             0x03,
//         ];

//         let (_cd_block, position) = CDBLOCK::read(&cd_data, true);

//         assert_eq!(position, 0);
//     }

//     #[test]
//     fn write() {}
// }

use crate::{utils, mdf3::signals::Signals};