use std::mem;

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
    #[allow(dead_code)]
    id_reserved1: [u8; 4],
    #[allow(dead_code)]
    id_ver: u16,
    #[allow(dead_code)]
    id_reserved2: [u8; 34],
}
impl Block for Idblock {
    fn new() -> Self {
        Self {
            id_file: [0; 8],
            id_vers: [0; 8],
            id_prog: [0; 8],
            id_reserved1: [0; 4],
            id_ver: 0,
            id_reserved2: [0; 34],
        }
    }
    fn default() -> Self {
        Self {
            id_file: [0; 8],
            id_vers: [0; 8],
            id_prog: [0; 8],
            id_reserved1: [0; 4],
            id_ver: 0,
            id_reserved2: [0; 34],
        }
    }
    fn read(stream: &[u8], _position: usize, _little_endian: bool) -> (usize, Self) {
        let mut pos = 0;
        let litte_endian = true;
        let id_file = utils::read(stream, _little_endian, &mut pos);
        let id_vers = utils::read(stream, litte_endian, &mut pos);
        let id_prog = utils::read(stream, litte_endian, &mut pos);
        let id_reserved1: [u8; 4] = utils::read(stream, litte_endian, &mut pos);
        let id_ver = utils::read(stream, litte_endian, &mut pos);
        let id_reserved2: [u8; 34] = utils::read(stream, litte_endian, &mut pos);

        (
            pos,
            Self {
                id_file,
                id_vers,
                id_prog,
                id_reserved1,
                id_ver,
                id_reserved2,
            },
        )
    }

	fn byte_len(&self) -> usize {
		mem::size_of_val(&self.id_file) +
		mem::size_of_val(&self.id_vers) +
		mem::size_of_val(&self.id_prog) +
		mem::size_of_val(&self.id_reserved1) +
		mem::size_of_val(&self.id_ver) +
		mem::size_of_val(&self.id_reserved2) 
	}
}

#[test]
fn id_read_test() {
    let raw: [u8; 64] = [
        0x4D, 0x44, 0x46, 0x20, 0x20, 0x20, 0x20, 0x20, 0x34, 0x2E, 0x31, 0x30, 0x20, 0x20, 0x20,
        0x20, 0x54, 0x47, 0x54, 0x20, 0x31, 0x35, 0x2E, 0x30, 0x00, 0x00, 0x00, 0x00, 0x9A, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    let (pos, id_result) = Idblock::read(&raw, 0, true);

    assert_eq!(64, pos);
    assert!(utils::eq("MDF     ".as_bytes(), &id_result.id_file));
    assert!(utils::eq("4.10    ".as_bytes(), &id_result.id_vers));
    assert!(utils::eq("TGT 15.0".as_bytes(), &id_result.id_prog));
    assert!(utils::eq(&[0_u8; 4], &id_result.id_reserved1));
    assert_eq!(410, id_result.id_ver);
    assert!(utils::eq(&[0_u8; 34], &id_result.id_reserved2));
}
