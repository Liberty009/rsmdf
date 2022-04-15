use super::{
    block::{Block, DataBlock},
    block_header::BlockHeader,
    dl_block::Dlblock,
    dt_block::Dtblock,
    dz_block::Dzblock,
};

pub enum DataBlockType {
    Block(Dtblock),
    BlockComp(Dzblock),
    List(Dlblock),
}

impl DataBlockType {
    pub fn data_array(&self) -> Vec<u8> {
        match self {
            Self::Block(block) => block.data_array(),
            Self::BlockComp(block) => block.data_array(),
            Self::List(block) => block.data_array(),
        }
    }

    pub fn read(stream: &[u8], position: usize, little_endian: bool) -> Self {
        let (_pos, header) = BlockHeader::read(stream, position, little_endian);

        let data_block = match std::str::from_utf8(&header.id).unwrap() {
            "##DT" => {
                let (_pos, block) = Dtblock::read(stream, position, little_endian);
                Self::Block(block)
            }
            "##DZ" => {
                let (_pos, block) = Dzblock::read(stream, position, little_endian);
                Self::BlockComp(block)
            }
            "##DL" => {
                let (_pos, block) = Dlblock::read(stream, position, little_endian);
                Self::List(block)
            }
            "##HL" => todo!(),
            _ => panic!("Error: wrong block type for data block"),
        };

        data_block
    }
}
