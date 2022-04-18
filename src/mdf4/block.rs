pub trait Block {
    fn new() -> Self;
    fn default() -> Self;
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self);
    fn byte_len(&self) -> usize;
    //fn is_empty(&self) -> bool;
}

pub trait LinkedBlock {
    fn next(&self, stream: &[u8], little_endian: bool) -> Option<Self>
    where
        Self: std::marker::Sized;
    fn list(&self, stream: &[u8], little_endian: bool) -> Vec<Self>
    where
        Self: std::marker::Sized;
}

pub trait DataBlock {
    fn data_array(&self, stream: &[u8], little_endian: bool) -> Vec<u8>;
}
