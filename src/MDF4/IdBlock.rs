use crate::utils;
use crate::MDF4::Block::Block;

#[derive(Debug, Clone)]
pub struct Idblock {
    #[allow(dead_code)]
    id_file: [u8; 8],
    #[allow(dead_code)]
    id_vers: [u8; 8],
    #[allow(dead_code)]
    id_prog: [u8; 8],

    //id_reserved1: [u8; 4],
    #[allow(dead_code)]
    id_ver: u16,
    //id_reserved2: [u8; 34],
}
impl Block for Idblock {
    fn new() -> Self {
        Self {
            id_file: [0; 8],
            id_vers: [0; 8],
            id_prog: [0; 8],
            //id_reserved1: [0; 4],
            id_ver: 0,
            //id_reserved2: [0; 34],
        }
    }
    fn default() -> Self {
        Self {
            id_file: [0; 8],
            id_vers: [0; 8],
            id_prog: [0; 8],
            //id_reserved1: [0; 4],
            id_ver: 0,
            //id_reserved2: [0; 34],
        }
    }
    fn read(stream: &[u8], _position: usize, _little_endian: bool) -> (usize, Self) {
        let mut pos = 0;
        let litte_endian = true;
        let id_file = utils::read(stream, _little_endian, &mut pos);
        let id_vers = utils::read(stream, litte_endian, &mut pos);
        let id_prog = utils::read(stream, litte_endian, &mut pos);
        let _id_reserved1: [u8; 4] = utils::read(stream, litte_endian, &mut pos);
        let id_ver = utils::read(stream, litte_endian, &mut pos);
        let _id_reserved2: [u8; 34] = utils::read(stream, litte_endian, &mut pos);

        (
            pos,
            Self {
                id_file,
                id_vers,
                id_prog,
                //id_reserved1,
                id_ver,
                //id_reserved2,
            },
        )
    }
}
