use std::mem;

use crate::utils;

#[derive(Debug, Clone, Copy)]
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

	pub fn is_empty(self) -> bool{
		self.len() == 0
	}
}

pub fn _print_record(value: Record) {
    match value {
        Record::Uint(number) => print!("{}", number),
        Record::Int(number) => print!("{}", number),
        Record::Float32(number) => print!("{}", number),
        Record::Float64(number) => print!("{}", number),
        // _ => panic!("Help!")
    };
}

pub enum Record {
    Uint(u8),
    Int(i8),
    Float32(f32),
    Float64(f64),
}

impl Record {
    pub fn new(stream: &[u8], dtype: DataTypeRead) -> Self {
        let rec = match dtype.data_type {
            DataType::UnsignedInt => Self::unsigned_int(stream, dtype),
            DataType::SignedInt => Self::signed_int(stream, dtype),
            DataType::Float32 => Self::float32(stream, dtype),
            DataType::Float64 => Self::float64(stream, dtype),
            _ => (panic!("Incorrect or not implemented type!")),
        };

        rec
    }

    pub fn extract(&self) -> f64 {
        match self {
            Record::Uint(number) => *number as f64,
            Record::Int(number) => *number as f64,
            Record::Float32(number) => *number as f64,
            Record::Float64(number) => *number as f64,
            // _ => panic!("Help!")
        }
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
