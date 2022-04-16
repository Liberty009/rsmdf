use crate::{utils, record::{DataTypeRead, DataType}};

use super::tx_block::Txblock;


const UNSIGNED_INT_DEFAULT: u16 = 0;
const SIGNED_INT_DEFAULT: u16 = 1;
const FLOAT32_DEFAULT: u16 = 2;
const FLOAT64_DEFAULT: u16 = 3;
const FFLOAT_DEFAULT: u16 = 4;
const GFLOAT_DEFAULT: u16 = 5;
const DFLOAT_DEFAULT: u16 = 6;
const STRING_NULL_TERM: u16 = 7;
const BYTE_ARRAY: u16 = 8;
const UNSIGNED_INT_BIGENDIAN: u16 = 9;
const SIGNED_INT_BIGENDIAN: u16 = 10;
const FLOAT32_BIGENDIAN: u16 = 11;
const FLOAT64_BIGENDIAN: u16 = 12;
const UNSIGNED_INT_LITTLEENDIAN: u16 = 13;
const SIGNED_INT_LITTLEENDIAN: u16 = 14;
const FLOAT32_INT_LITTLEENDIAN: u16 = 15;
const FLOAT64_INT_LITTLEENDIAN: u16 = 16;

#[derive(Debug, Clone, Copy)]
pub struct Cnblock {
    pub block_type: [u8; 2],
    pub block_size: u16,
    pub next: u32,
    pub conversion_formula: u32,
    pub source_ext: u32,
    pub dependency: u32,
    pub comment: u32,
    pub channel_type: u16,
    pub short_name: [u8; 32],
    pub desc: [u8; 128],
    pub start_offset: u16,
    pub bit_number: u16,
    pub data_type: DataTypeRead,
    pub value_range_valid: u16,
    pub signal_min: f64,
    pub signal_max: f64,
    pub sample_rate: f64,
    pub long_name: u32,
    pub display_name: u32,
    pub addition_byte_offset: u16,
}

impl Cnblock {
    pub fn write() {}
    pub fn read(stream: &[u8], little_endian: bool, position: usize) -> (Cnblock, usize) {
        let mut pos = position;
        let block_type: [u8; 2] = stream[pos..pos + 2].try_into().expect("msg");
        pos += block_type.len();
        if !utils::eq(&block_type, &[b'C', b'N']) {
            panic!("CNBLOCK not found.");
        }

        let block_size = utils::read(stream, little_endian, &mut pos);
        let next = utils::read(stream, little_endian, &mut pos);
        let conversion_formula = utils::read(stream, little_endian, &mut pos);
        let source_ext = utils::read(stream, little_endian, &mut pos);
        let dependency = utils::read(stream, little_endian, &mut pos);
        let comment = utils::read(stream, little_endian, &mut pos);
        let channel_type = utils::read(stream, little_endian, &mut pos);

        let short_name: [u8; 32] = stream[pos..pos + 32].try_into().expect("msg");
        pos += short_name.len();

        let desc: [u8; 128] = stream[pos..pos + 128].try_into().expect("msg");
        pos += desc.len();

        let start_offset = utils::read(stream, little_endian, &mut pos);
        let bit_number = utils::read(stream, little_endian, &mut pos);

        let datatype: u16 = utils::read(stream, little_endian, &mut pos);
        let data_type = match datatype {
            UNSIGNED_INT_DEFAULT => DataTypeRead {
                data_type: DataType::UnsignedInt,
                little_endian,
            },
            SIGNED_INT_DEFAULT => DataTypeRead {
                data_type: DataType::SignedInt,
                little_endian,
            },
            FLOAT32_DEFAULT => DataTypeRead {
                data_type: DataType::Float32,
                little_endian,
            },
            FLOAT64_DEFAULT => DataTypeRead {
                data_type: DataType::Float64,
                little_endian,
            },
            FFLOAT_DEFAULT => DataTypeRead {
                data_type: DataType::FFloat,
                little_endian,
            },
            GFLOAT_DEFAULT => DataTypeRead {
                data_type: DataType::GFloat,
                little_endian,
            },
            DFLOAT_DEFAULT => DataTypeRead {
                data_type: DataType::DFloat,
                little_endian,
            },
            STRING_NULL_TERM => DataTypeRead {
                data_type: DataType::StringNullTerm,
                little_endian,
            },
            BYTE_ARRAY => DataTypeRead {
                data_type: DataType::ByteArray,
                little_endian,
            },
            UNSIGNED_INT_BIGENDIAN => DataTypeRead {
                data_type: DataType::UnsignedInt,
                little_endian: false,
            },
            SIGNED_INT_BIGENDIAN => DataTypeRead {
                data_type: DataType::SignedInt,
                little_endian: false,
            },
            FLOAT32_BIGENDIAN => DataTypeRead {
                data_type: DataType::Float32,
                little_endian: false,
            },
            FLOAT64_BIGENDIAN => DataTypeRead {
                data_type: DataType::Float64,
                little_endian: false,
            },
            UNSIGNED_INT_LITTLEENDIAN => DataTypeRead {
                data_type: DataType::UnsignedInt,
                little_endian: true,
            },
            SIGNED_INT_LITTLEENDIAN => DataTypeRead {
                data_type: DataType::SignedInt,
                little_endian: true,
            },
            FLOAT32_INT_LITTLEENDIAN => DataTypeRead {
                data_type: DataType::Float32,
                little_endian: true,
            },
            FLOAT64_INT_LITTLEENDIAN => DataTypeRead {
                data_type: DataType::Float64,
                little_endian: true,
            },
            _ => {
                println!("Found data type: {}", datatype);
                panic!("Data type not found. Type was:")
            }
        };

        let value_range_valid = utils::read(stream, little_endian, &mut pos);
        let signal_min = utils::read(stream, little_endian, &mut pos);
        let signal_max = utils::read(stream, little_endian, &mut pos);
        let sample_rate = utils::read(stream, little_endian, &mut pos);
        let long_name = utils::read(stream, little_endian, &mut pos);
        let display_name = utils::read(stream, little_endian, &mut pos);
        let addition_byte_offset = utils::read(stream, little_endian, &mut pos);

        (
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
            pos,
        )
    }

    pub fn name(self, stream: &[u8], little_endian: bool) -> String {
        let mut name = "".to_string();

        if self.channel_type == 1 {
            name = "time".to_string();
        } else if self.long_name != 0 {
            let (tx, _pos) = Txblock::read(stream, self.long_name as usize, little_endian);

            name = match std::str::from_utf8(&tx.text) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
        }

        name
    }
}

#[cfg(test)]
mod cnblock_test {
    use crate::utils;

    use super::*;

    #[test]
    fn read() {
        let cn_data = [
            0x43, 0x4E, 0xE4, 0x00, 0xA6, 0xE3, 0x10, 0x00, 0x80, 0xE0, 0x10, 0x00, 0xAE, 0xE0,
            0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x74, 0x69,
            0x6D, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x03, 0x00, 0x01, 0x00, 0x04, 0x19,
            0x60, 0x9C, 0xAE, 0xDD, 0xBC, 0x3F, 0x52, 0xE8, 0x62, 0xFA, 0x56, 0xD3, 0x28, 0x40,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x54, 0x58, 0x2B, 0x00, 0x41, 0x53, 0x41, 0x4D, 0x2E, 0x4D,
            0x2E, 0x53, 0x43, 0x41, 0x4C, 0x41, 0x52, 0x2E, 0x53, 0x42, 0x59, 0x54, 0x45, 0x2E,
            0x49, 0x44, 0x45, 0x4E, 0x54, 0x49, 0x43, 0x41, 0x4C, 0x2E, 0x44, 0x49, 0x53, 0x43,
            0x52, 0x45, 0x54, 0x45, 0x00, 0x54, 0x58, 0xBB,
        ];

        let (cn_block, _position) = Cnblock::read(&cn_data, true, 0);

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
            (cn_block.signal_min - utils::read::<f64>(
                &[0x04, 0x19, 0x60, 0x9C, 0xAE, 0xDD, 0xBC, 0x3F,],
                true,
                &mut 0_usize)).abs() < 0.1
        );
        assert!(
            (cn_block.signal_max-
            utils::read::<f64>(
                &[0x52, 0xE8, 0x62, 0xFA, 0x56, 0xD3, 0x28, 0x40,],
                true,
                &mut 0_usize
            )).abs() < 0.1
        );
        assert!((cn_block.sample_rate - 0.0).abs() < 0.1);

        assert_eq!(cn_block.display_name, 0);
        assert_eq!(cn_block.addition_byte_offset, 0);
    }

    #[test]
    fn write() {}
}
