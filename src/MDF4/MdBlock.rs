#[derive(Debug, Clone)]
struct Mdblock {
    #[allow(dead_code)]
    md_data: String,
}
impl Block for Mdblock {
    fn new() -> Self {
        Self {
            md_data: "".to_string(),
        }
    }
    fn default() -> Self {
        Self {
            md_data: "".to_string(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##MD".as_bytes()) {
            panic!("Error type incorrect");
        }

        let md_data: String = unsafe {
            str_from_u8_nul_utf8_unchecked(&stream[pos..(pos + header.length as usize - 10)])
                .to_string()
        };

        (pos + md_data.len(), Self { md_data })
    }
}