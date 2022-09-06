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

    display_name
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
        read_le(&input[old..])
    } else {
        read_be(&input[old..])
    }
}

pub fn eq(array1: &[u8], other: &[u8]) -> bool {
    array1.iter().zip(other.iter()).all(|(a, b)| a == b)
}
