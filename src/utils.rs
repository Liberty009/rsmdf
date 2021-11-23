use std::mem;
use xml::reader::{EventReader, XmlEvent};

pub fn extract_name(text: &[u8]) -> String {
    let parser = EventReader::new(text);

    let mut display_name = "".to_string();

    let mut display = false;
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                //println!("{}", name);
                if name.to_string().contains("display") {
                    display = true;
                }
            }
            Ok(XmlEvent::Characters(text)) => {
                if display {
                    //println!("{}", text); // or something else
                    display_name = text;
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string().contains("display") {
                    display = false;
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    return display_name;
}

pub trait FromBytes {
    fn from_be_bytes(a: &[u8]) -> Self;
    fn from_le_bytes(a: &[u8]) -> Self;
}

impl<const N: usize> FromBytes for [u8; N] {
    fn from_be_bytes(a: &[u8]) -> [u8; N] {
        let (int_bytes, _rest) = a.split_at(N);

        let mut me = [0u8; N];
        me.copy_from_slice(int_bytes);

        //*a = rest;
        me
    }
    fn from_le_bytes(a: &[u8]) -> [u8; N] {
        let (int_bytes, _rest) = a.split_at(N);

        let mut me = [0u8; N];
        me.copy_from_slice(int_bytes);

        //*a = rest;
        me
    }
}

impl FromBytes for u64 {
    fn from_be_bytes(a: &[u8]) -> Self {
        Self::from_be_bytes(FromBytes::from_be_bytes(a))
    }
    fn from_le_bytes(a: &[u8]) -> Self {
        Self::from_le_bytes(FromBytes::from_le_bytes(a))
    }
}
impl FromBytes for u32 {
    fn from_be_bytes(a: &[u8]) -> Self {
        Self::from_be_bytes(FromBytes::from_be_bytes(a))
    }
    fn from_le_bytes(a: &[u8]) -> Self {
        Self::from_le_bytes(FromBytes::from_le_bytes(a))
    }
}
impl FromBytes for u16 {
    fn from_be_bytes(a: &[u8]) -> Self {
        Self::from_be_bytes(FromBytes::from_be_bytes(a))
    }
    fn from_le_bytes(a: &[u8]) -> Self {
        Self::from_le_bytes(FromBytes::from_le_bytes(a))
    }
}
impl FromBytes for u8 {
    fn from_be_bytes(a: &[u8]) -> Self {
        Self::from_be_bytes(FromBytes::from_be_bytes(a))
    }
    fn from_le_bytes(a: &[u8]) -> Self {
        Self::from_le_bytes(FromBytes::from_le_bytes(a))
    }
}

impl FromBytes for i64 {
    fn from_be_bytes(a: &[u8]) -> Self {
        Self::from_be_bytes(FromBytes::from_be_bytes(a))
    }
    fn from_le_bytes(a: &[u8]) -> Self {
        Self::from_le_bytes(FromBytes::from_le_bytes(a))
    }
}
impl FromBytes for i32 {
    fn from_be_bytes(a: &[u8]) -> Self {
        Self::from_be_bytes(FromBytes::from_be_bytes(a))
    }
    fn from_le_bytes(a: &[u8]) -> Self {
        Self::from_le_bytes(FromBytes::from_le_bytes(a))
    }
}
impl FromBytes for i16 {
    fn from_be_bytes(a: &[u8]) -> Self {
        Self::from_be_bytes(FromBytes::from_be_bytes(a))
    }
    fn from_le_bytes(a: &[u8]) -> Self {
        Self::from_le_bytes(FromBytes::from_le_bytes(a))
    }
}
impl FromBytes for i8 {
    fn from_be_bytes(a: &[u8]) -> Self {
        Self::from_be_bytes(FromBytes::from_be_bytes(a))
    }
    fn from_le_bytes(a: &[u8]) -> Self {
        Self::from_le_bytes(FromBytes::from_le_bytes(a))
    }
}

impl FromBytes for f64 {
    fn from_be_bytes(a: &[u8]) -> Self {
        Self::from_be_bytes(FromBytes::from_be_bytes(a))
    }
    fn from_le_bytes(a: &[u8]) -> Self {
        Self::from_le_bytes(FromBytes::from_le_bytes(a))
    }
}
impl FromBytes for f32 {
    fn from_be_bytes(a: &[u8]) -> Self {
        Self::from_be_bytes(FromBytes::from_be_bytes(a))
    }
    fn from_le_bytes(a: &[u8]) -> Self {
        Self::from_le_bytes(FromBytes::from_le_bytes(a))
    }
}

pub fn read_be<T: FromBytes>(input: &[u8]) -> T {
    T::from_be_bytes(input)
}
pub fn read_le<T: FromBytes>(input: &[u8]) -> T {
    T::from_le_bytes(input)
}
pub fn read<T: FromBytes>(input: &[u8], little_endian: bool, position: &mut usize) -> T {
    let old = *position;
    *position += mem::size_of::<T>() / mem::size_of::<u8>();
    if little_endian {
        return read_le(&input[old..]);
    } else {
        return read_be(&input[old..]);
    }
}

// pub fn read<T>(stream: &[u8], _little_endian: bool, position: &mut usize) -> [T]
// where T: Copy + FromByteSlice
// {

// 	*position += mem::size_of::<T>() / mem::size_of::<u8>();
//     // if little_endian {
//     //     return LittleEndian::read::<T>(stream);
//     // } else {
//     //     return BigEndian::read::<T>>(stream);
//     // }

// 	let values = T::from_byte_slice(&stream).expect("");

// 	// if little_endian {
// 		return values;
// 	// } else {
// 	// 	return T::from_be_bytes(stream).expect("");
// 	// }
// }

// pub fn read_u8(stream: &[u8], _little_endian: bool, position: &mut usize) -> u8 {
//     *position += 1;
//     return stream[0];
// }

// pub fn read_i8(stream: &[u8], _little_endian: bool, position: &mut usize) -> i8 {
//     *position += 1;
//     return unsafe { std::mem::transmute::<u8, i8>(stream[0]) };
// }

// pub fn read_u16(stream: &[u8], little_endian: bool, position: &mut usize) -> u16 {
//     *position += mem::size_of::<u16>() / mem::size_of::<u8>();
//     if little_endian {
//         return LittleEndian::read_u16(stream);
//     } else {
//         return BigEndian::read_u16(stream);
//     }
// }

// pub fn read_i16(stream: &[u8], little_endian: bool, position: &mut usize) -> i16 {
//     *position += mem::size_of::<i16>() / mem::size_of::<u8>();
//     if little_endian {
//         return LittleEndian::read_i16(stream);
//     } else {
//         return BigEndian::read_i16(stream);
//     }
// }

// pub fn read_u32(stream: &[u8], little_endian: bool, position: &mut usize) -> u32 {
//     *position += mem::size_of::<u32>() / mem::size_of::<u8>();

//     if little_endian {
//         return LittleEndian::read_u32(stream);
//     } else {
//         return BigEndian::read_u32(stream);
//     }
// }

// pub fn read_u64(stream: &[u8], little_endian: bool, position: &mut usize) -> u64 {
//     *position += mem::size_of::<u64>() / mem::size_of::<u8>();

//     if little_endian {
//         return LittleEndian::read_u64(stream);
//     } else {
//         return BigEndian::read_u64(stream);
//     }
// }

// pub fn read_f64(stream: &[u8], little_endian: bool, position: &mut usize) -> f64 {
//     *position += mem::size_of::<f64>() / mem::size_of::<u8>();
//     if little_endian {
//         return LittleEndian::read_f64(stream);
//     } else {
//         return BigEndian::read_f64(stream);
//     }
// }

pub fn eq(array1: &[u8], other: &[u8]) -> bool {
    array1.iter().zip(other.iter()).all(|(a, b)| a == b)
}
