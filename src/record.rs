use std::mem;

use crate::utils;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    UnsignedInt,
    SignedInt,
    Float32,
    Float64,
    FFloat,
    GFloat,
    DFloat,
    StringNullTerm,
    ByteArray,
}

#[derive(Debug, Clone, Copy)]
pub struct DataTypeRead {
    pub data_type: DataType,
    pub little_endian: bool,
}

impl DataTypeRead {
    pub fn new(datatype: u16, little_endian: bool) -> Self {
        match datatype {
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
        }
    }

    pub fn write(&self, _little_endian: bool) -> Vec<u8> {
        match self.data_type {
            DataType::UnsignedInt => vec![
                UNSIGNED_INT_DEFAULT.try_into().unwrap(),
                self.little_endian.try_into().unwrap(),
            ],
            DataType::SignedInt => vec![
                SIGNED_INT_DEFAULT.try_into().unwrap(),
                self.little_endian.try_into().unwrap(),
            ],
            DataType::Float32 => vec![
                FLOAT32_DEFAULT.try_into().unwrap(),
                self.little_endian.try_into().unwrap(),
            ],
            DataType::Float64 => vec![
                FLOAT64_DEFAULT.try_into().unwrap(),
                self.little_endian.try_into().unwrap(),
            ],
            DataType::FFloat => vec![
                FFLOAT_DEFAULT.try_into().unwrap(),
                self.little_endian.try_into().unwrap(),
            ],
            DataType::GFloat => vec![
                GFLOAT_DEFAULT.try_into().unwrap(),
                self.little_endian.try_into().unwrap(),
            ],
            DataType::DFloat => vec![
                DFLOAT_DEFAULT.try_into().unwrap(),
                self.little_endian.try_into().unwrap(),
            ],
            DataType::StringNullTerm => vec![
                STRING_NULL_TERM.try_into().unwrap(),
                self.little_endian.try_into().unwrap(),
            ],
            DataType::ByteArray => vec![
                BYTE_ARRAY.try_into().unwrap(),
                self.little_endian.try_into().unwrap(),
            ],
        }
    }

    pub fn len(self) -> usize {
        match self.data_type {
            DataType::UnsignedInt => mem::size_of::<u8>() / mem::size_of::<u8>(),
            DataType::SignedInt => mem::size_of::<i8>() / mem::size_of::<u8>(),
            DataType::Float32 => mem::size_of::<f32>() / mem::size_of::<u8>(),
            DataType::Float64 => mem::size_of::<f64>() / mem::size_of::<u8>(),
            DataType::FFloat => 0,
            DataType::GFloat => 0,
            DataType::DFloat => 0,
            DataType::StringNullTerm => 0,
            DataType::ByteArray => 0,
            // _ => panic!("")
        }
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
}

pub fn _print_record(value: Record) {
    match value {
        Record::Uint(number) => print!("{}", number),
        Record::Int(number) => print!("{}", number),
        Record::Float32(number) => print!("{}", number),
        Record::Float64(number) => print!("{}", number),
        Record::StringNullTerm(string) => print!("{}", string),
        // _ => panic!("Help!")
    };
}

#[derive(Debug, PartialEq, Clone)]
pub enum Record {
    Uint(u8),
    Int(i8),
    Float32(f32),
    Float64(f64),
    StringNullTerm(String),
}

impl Record {
    pub fn new(stream: &[u8], dtype: DataTypeRead) -> Self {
        match dtype.data_type {
            DataType::UnsignedInt => Self::unsigned_int(stream, dtype),
            DataType::SignedInt => Self::signed_int(stream, dtype),
            DataType::Float32 => Self::float32(stream, dtype),
            DataType::Float64 => Self::float64(stream, dtype),
            DataType::StringNullTerm => Self::string_null_term(stream, dtype),
            _ => panic!("Incorrect or not implemented type!, {:?}", dtype.data_type),
        }
    }

    pub fn write(&self, little_endian: bool) -> Vec<u8> {
        let mut array = Vec::new();
        match self {
            Record::Uint(number) => array.push(*number),
            Record::Int(number) => array.push(*number as u8),
            Record::Float32(number) => array.append(&mut utils::write(*number, little_endian)),
            Record::Float64(number) => array.append(&mut utils::write(*number, little_endian)),
            Record::StringNullTerm(string) => array.append(&mut string.clone().into_bytes()),
        }

        array
    }

    pub fn extract(&self) -> f64 {
        match self {
            Record::Uint(number) => *number as f64,
            Record::Int(number) => *number as f64,
            Record::Float32(number) => *number as f64,
            Record::Float64(number) => *number,
            Record::StringNullTerm(string) => string.parse::<f64>().unwrap(),
        }
    }

    fn string_null_term(stream: &[u8], _dtype: DataTypeRead) -> Self {
        let mut string = String::new();

        for char in stream {
            if *char == 0 {
                break;
            }
            string.push(*char as char);
        }
        Record::StringNullTerm(string)
    }

    fn unsigned_int(stream: &[u8], dtype: DataTypeRead) -> Self {
        let records = utils::read(stream, dtype.little_endian, &mut 0);

        Self::Uint(records)
    }

    fn signed_int(stream: &[u8], dtype: DataTypeRead) -> Self {
        let records = utils::read(stream, dtype.little_endian, &mut 0);

        Self::Int(records)
    }

    fn float32(stream: &[u8], dtype: DataTypeRead) -> Self {
        let records = utils::read(stream, dtype.little_endian, &mut 0);

        Self::Float32(records)
    }
    fn float64(stream: &[u8], dtype: DataTypeRead) -> Self {
        let records = utils::read(stream, dtype.little_endian, &mut 0);

        Self::Float64(records)
    }
}
