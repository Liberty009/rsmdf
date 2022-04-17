#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConversionData {
    Parameters,
    Table,
    #[allow(dead_code)]
    Text,
}

impl ConversionData {
    #[allow(dead_code)]
    pub fn write() {}
    pub fn read(_data: &[u8], _little_endian: bool, datatype: u8) -> (ConversionData, usize) {
        if datatype == 1 {
            (ConversionData::Parameters, 1)
        } else {
            (ConversionData::Table, 1)
        }
    }
}
