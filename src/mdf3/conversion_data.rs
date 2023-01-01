#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionData {
    Parameters,
    Table,
    Text,
}

impl ConversionData {
    pub fn write() {}
    pub fn read(_data: &[u8], _little_endian: bool, datatype: u8) -> (ConversionData, usize) {
        if datatype == 1 {
            (ConversionData::Parameters, 1)
        } else {
            (ConversionData::Table, 1)
        }
    }
}
