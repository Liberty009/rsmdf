pub trait Mdf3Block {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self);
}

pub trait LinkedBlock {
    fn next(&self, stream: &[u8], little_endian: bool) -> Option<Self>
    where
        Self: std::marker::Sized;
    fn list(&self, stream: &[u8], little_endian: bool) -> Vec<Self>
    where
        Self: std::marker::Sized;
}
