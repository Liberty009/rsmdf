use super::block::Block;
use super::block_header::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Sdblock {
    header: BlockHeader,
    sd_data: Vec<u8>,
}

impl Block for Sdblock {
    fn new() -> Self {
        Self {
            header: BlockHeader::create("##SD", 24, 0),
            sd_data: Vec::new(),
        }
    }

    fn default() -> Self {
        Self {
            header: BlockHeader::new(),
            sd_data: Vec::new(),
        }
    }

    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        let sd_data = stream[pos..(pos + header.length as usize - header.byte_len())].to_vec();

        let pos = pos + sd_data.len();

        (pos, Self { header, sd_data })
    }

    fn byte_len(&self) -> usize {
        self.header.byte_len() + 
        self.sd_data.len()
    }
}
