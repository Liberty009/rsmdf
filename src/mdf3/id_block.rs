use crate::utils;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Idblock {
    file_id: [u8; 8],
    format_id: [u8; 8],
    program_id: [u8; 8],
    default_byte_order: u16,
    default_float_format: u16,
    version_number: u16,
    code_page_number: u16,
    reserved1: [u8; 2],
    reserved2: [u8; 30],
}

impl Idblock {
    #[allow(dead_code)]
    pub fn write(&self, little_endian: bool) -> Vec<u8> {
        let mut array = Vec::new();

        array.append(&mut self.file_id.to_vec());
        array.append(&mut self.format_id.to_vec());
        array.append(&mut self.program_id.to_vec());
        array.append(&mut utils::write(self.default_byte_order, little_endian));
        array.append(&mut utils::write(self.default_float_format, little_endian));
        array.append(&mut utils::write(self.version_number, little_endian));
        array.append(&mut utils::write(self.code_page_number, little_endian));
        array.append(&mut self.reserved1.to_vec());
        array.append(&mut self.reserved2.to_vec());

        array
    }
    #[allow(dead_code)]
    pub fn new(
        file_id: [u8; 8],
        format_id: [u8; 8],
        program_id: [u8; 8],
        default_byte_order: u16,
        default_float_format: u16,
        version_number: u16,
        code_page_number: u16,
    ) -> Self {
        Self {
            file_id,
            format_id,
            program_id,
            default_byte_order,
            default_float_format,
            version_number,
            code_page_number,
            reserved1: [0_u8; 2],
            reserved2: [0_u8; 30],
        }
    }
    pub fn default() -> Self {
        Self::new([0u8; 8], [0u8; 8], [0u8; 8], 0u16, 0u16, 0u16, 0u16)
    }
    pub fn read(stream: &[u8]) -> (Idblock, usize, bool) {
        let mut position = 0;
        let file_id: [u8; 8] = stream[position..position + 8].try_into().expect("msg");

        if !utils::eq(
            &file_id[..],
            &[0x4D, 0x44, 0x46, 0x20, 0x20, 0x20, 0x20, 0x20],
        ) {
            panic!("Error: Incorrect file type");
        }

        position += file_id.len();

        let format_id: [u8; 8] = stream[position..position + 8].try_into().expect("msg");
        position += format_id.len();

        let program_id: [u8; 8] = stream[position..position + 8].try_into().expect("msg");
        position += program_id.len();

        let default_byte_order = utils::read(stream, true, &mut position);

        let little_endian = default_byte_order == 0;

        let default_float_format = utils::read(stream, little_endian, &mut position);

        let version_number = utils::read(stream, little_endian, &mut position);

        let code_page_number = utils::read(stream, little_endian, &mut position);

        let reserved1: [u8; 2] = [stream[position], stream[position + 1]];
        position += reserved1.len();
        let reserved2: [u8; 30] = stream[position..position + 30].try_into().expect("msg");
        position += reserved2.len();

        (
            Idblock {
                file_id,
                format_id,
                program_id,
                default_byte_order,
                default_float_format,
                version_number,
                code_page_number,
                reserved1,
                reserved2,
            },
            position,
            little_endian,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read() {
        let id_data = [
            0x4D, 0x44, 0x46, 0x20, 0x20, 0x20, 0x20, 0x20, 0x33, 0x2E, 0x33, 0x30, 0x00, 0x00,
            0x00, 0x00, 0x61, 0x6D, 0x64, 0x66, 0x36, 0x34, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x4A, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let (id_block, position, endian) = Idblock::read(&id_data);

        assert_eq!(position, 64);
        assert!(endian);
        assert!(utils::eq(
            &id_block.format_id,
            &[0x33, 0x2E, 0x33, 0x30, 0x00, 0x00, 0x00, 0x00,]
        ));
        assert!(utils::eq(
            &id_block.program_id,
            &[0x61, 0x6D, 0x64, 0x66, 0x36, 0x34, 0x34, 0x00,]
        ));
        assert_eq!(id_block.default_float_format, 0);
        assert_eq!(id_block.version_number, 330);
        assert_eq!(id_block.code_page_number, 0);
        assert!(utils::eq(&id_block.reserved1, &[0, 0]));
        assert!(utils::eq(
            &id_block.reserved2,
            &[
                00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00,
                00, 00, 00, 00, 00, 00, 00, 00, 00,
            ]
        ))
    }

    #[test]
    fn write() {}
}
