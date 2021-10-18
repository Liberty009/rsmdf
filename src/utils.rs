use std::mem;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

pub fn read_u8(stream: &[u8], _little_endian: bool, position: &mut usize) -> u8 {
    *position += 1;
    return stream[0];
}

pub fn read_u16(stream: &[u8], little_endian: bool, position: &mut usize) -> u16 {
    *position += mem::size_of::<u16>() / mem::size_of::<u8>();
    if little_endian {
        return LittleEndian::read_u16(stream);
    } else {
        return BigEndian::read_u16(stream);
    }
}

pub fn read_i16(stream: &[u8], little_endian: bool, position: &mut usize) -> i16 {
    *position += mem::size_of::<i16>() / mem::size_of::<u8>();
    if little_endian {
        return LittleEndian::read_i16(stream);
    } else {
        return BigEndian::read_i16(stream);
    }
}

pub fn read_u32(stream: &[u8], little_endian: bool, position: &mut usize) -> u32 {
    *position += mem::size_of::<u32>() / mem::size_of::<u8>();

    if little_endian {
        return LittleEndian::read_u32(stream);
    } else {
        return BigEndian::read_u32(stream);
    }
}

pub fn read_u64(stream: &[u8], little_endian: bool, position: &mut usize) -> u64 {
    *position += mem::size_of::<u64>() / mem::size_of::<u8>();

    if little_endian {
        return LittleEndian::read_u64(stream);
    } else {
        return BigEndian::read_u64(stream);
    }
}

pub fn read_f64(stream: &[u8], little_endian: bool, position: &mut usize) -> f64 {
    *position += mem::size_of::<f64>() / mem::size_of::<u8>();
    if little_endian {
        return LittleEndian::read_f64(stream);
    } else {
        return BigEndian::read_f64(stream);
    }
}
