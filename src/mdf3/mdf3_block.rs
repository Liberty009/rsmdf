pub trait Mdf3Block {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self);
    fn write(&self, start_position: usize, little_endian: bool) -> Vec<u8>;
}

pub trait LinkedBlock {
    fn next(&self, stream: &[u8], little_endian: bool) -> Option<Self>
    where
        Self: std::marker::Sized;
    fn list(&self, stream: &[u8], little_endian: bool) -> Vec<Self>
    where
        Self: std::marker::Sized;
}
