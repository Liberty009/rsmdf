use super::conversion_data::ConversionData;
use super::mdf3_block::Mdf3Block;
use crate::utils;

#[derive(Debug, Clone, PartialEq)]
pub struct Ccblock {
    block_type: [u8; 2],
    block_size: u16,
    physical_range_valid: u16,
    physical_min: f64,
    physical_max: f64,
    unit: [u8; 20],
    conversion_type: u16,
    size_info: u16,
    conversion_data: ConversionData,
}

impl Mdf3Block for Ccblock {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let block_type: [u8; 2] = utils::read(stream, little_endian, &mut pos);

        if !utils::eq(&block_type, "CC".as_bytes()) {
            panic!("CC not found");
        }

        let block_size = utils::read(stream, little_endian, &mut pos);
        let physical_range_valid = utils::read(stream, little_endian, &mut pos);
        let physical_min = utils::read(stream, little_endian, &mut pos);
        let physical_max = utils::read(stream, little_endian, &mut pos);
        let unit = utils::read(stream, little_endian, &mut pos);
        let conversion_type = utils::read(stream, little_endian, &mut pos);
        let size_info = utils::read(stream, little_endian, &mut pos);

        let datatype = 1;

        let (conversion_data, pos_conversion) =
            ConversionData::read(stream, little_endian, datatype);
        pos += pos_conversion;

        (
            pos,
            Self {
                block_type,
                block_size,
                physical_range_valid,
                physical_min,
                physical_max,
                unit,
                conversion_type,
                size_info,
                conversion_data,
            },
        )
    }

    fn write(&self, _start_position: usize, little_endian: bool) -> Vec<u8> {
        let mut array = self.block_type.to_vec();
        array.append(&mut utils::write(self.block_size, little_endian));
        array.append(&mut utils::write(self.physical_range_valid, little_endian));
        array.append(&mut utils::write(self.physical_min, little_endian));
        array.append(&mut utils::write(self.physical_max, little_endian));
        array.append(&mut self.unit.to_vec());
        array.append(&mut utils::write(self.conversion_type, little_endian));
        array.append(&mut utils::write(self.size_info, little_endian));
        //conversion_data: ConversionData,

        array
    }
}

impl Ccblock {}

#[cfg(test)]
mod tests {
    use crate::utils;

    use super::*;

    static CC_DATA: [u8; 233] = [
        0x43, 0x43, 0x2E, 0x00, 0x01, 0x00, 0x04, 0x19, 0x60, 0x9C, 0xAE, 0xDD, 0xBC, 0x3F, 0x52,
        0xE8, 0x62, 0xFA, 0x56, 0xD3, 0x28, 0x40, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00,
        0x00, 0x43, 0x45, 0x80, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x43, 0x68,
        0x61, 0x6E, 0x6E, 0x65, 0x6C, 0x20, 0x69, 0x6E, 0x73, 0x65, 0x72, 0x74, 0x65, 0x64, 0x20,
        0x62, 0x79, 0x20, 0x50, 0x79, 0x74, 0x68, 0x6F, 0x6E, 0x20, 0x53, 0x63, 0x72, 0x69, 0x70,
        0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x43, 0x4E, 0xE4, 0x00, 0xA6, 0xE3,
        0x10, 0x00, 0x80, 0xE0, 0x10, 0x00, 0xAE, 0xE0, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x74, 0x69, 0x6D, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn read() {
        let (position, cc_block) = Ccblock::read(&CC_DATA, 0, true);

        assert_eq!(position, 47); // should match the block size
        assert_eq!(cc_block.block_size, 46);
        assert_eq!(cc_block.physical_range_valid, 1);

        assert!(
            (cc_block.physical_min
                - utils::read::<f64>(
                    &[0x04, 0x19, 0x60, 0x9C, 0xAE, 0xDD, 0xBC, 0x3F],
                    true,
                    &mut 0_usize
                ))
            .abs()
                < 0.1
        );
        assert!(
            (cc_block.physical_max
                - utils::read::<f64>(
                    &[0x52, 0xE8, 0x62, 0xFA, 0x56, 0xD3, 0x28, 0x40],
                    true,
                    &mut 0_usize
                ))
            .abs()
                < 0.1
        );
        assert!(utils::eq(
            &cc_block.unit,
            &[
                0x73, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        ));

        assert_eq!(cc_block.conversion_type, 65535);
        assert_eq!(cc_block.size_info, 0);
        // assert_eq!(conversion_data: ConversionData,);
    }

    #[test]
    fn write() {
        let (_position, cc_block) = Ccblock::read(&CC_DATA, 0, true);

        let write_result = cc_block.write(0, true);

        assert!(utils::eq(&CC_DATA, &write_result));
    }
}
