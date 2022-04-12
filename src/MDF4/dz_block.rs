use super::block::Block;
use super::block_header::*;

#[derive(Debug, Clone)]
struct DZBlock {
    #[allow(dead_code)]
    dz_org_block_type: [u8; 2],
    #[allow(dead_code)]
    dz_zip_type: ZipType,
    //dz_reserved: u8,
    #[allow(dead_code)]
    dz_zip_parameter: u32,
    #[allow(dead_code)]
    dz_org_data_length: u64,
    #[allow(dead_code)]
    dz_data_length: u64,
    #[allow(dead_code)]
    dz_data: Vec<u8>,
}
impl Block for DZBlock {
    fn new() -> Self {
        Self {
            dz_org_block_type: [0_u8; 2],
            dz_zip_type: ZipType::Deflate,
            //dz_reserved: 0_u8,
            dz_zip_parameter: 0_u32,
            dz_org_data_length: 0_u64,
            dz_data_length: 0_u64,
            dz_data: Vec::new(),
        }
    }
    fn default() -> Self {
        Self {
            dz_org_block_type: [0_u8; 2],
            dz_zip_type: ZipType::Deflate,
            //dz_reserved: 0_u8,
            dz_zip_parameter: 0_u32,
            dz_org_data_length: 0_u64,
            dz_data_length: 0_u64,
            dz_data: Vec::new(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (mut pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##DZ".as_bytes()) {
            panic!("Error DZBLOCK");
        }

        let dz_org_block_type = utils::read(stream, little_endian, &mut pos);
        let dz_zip_type = ZipType::new(utils::read(stream, little_endian, &mut pos));
        let _dz_reserved: u8 = utils::read(stream, little_endian, &mut pos);
        let dz_zip_parameter = utils::read(stream, little_endian, &mut pos);
        let dz_org_data_length = utils::read(stream, little_endian, &mut pos);
        let dz_data_length = utils::read(stream, little_endian, &mut pos);
        let dz_data = stream[pos..pos + dz_data_length as usize].to_vec();

        pos += dz_data.len();

        (
            pos,
            Self {
                dz_org_block_type,
                dz_zip_type,
                //dz_reserved,
                dz_zip_parameter,
                dz_org_data_length,
                dz_data_length,
                dz_data,
            },
        )
    }
}