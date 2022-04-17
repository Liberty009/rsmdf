use crate::utils;

#[derive(Debug, Clone, Copy)]
pub struct ConversionLinear {
    pub p1: f64,
    pub p2: f64,
}

impl ConversionLinear {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionLinear, usize) {
        let mut position = 0;
        let p1 = utils::read(stream, little_endian, &mut position);
        let p2 = utils::read(stream, little_endian, &mut position);

        (ConversionLinear { p1, p2 }, position)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionPoly {
    pub p1: f64,
    pub p2: f64,
    pub p3: f64,
    pub p4: f64,
    pub p5: f64,
    pub p6: f64,
}

impl ConversionPoly {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionPoly, usize) {
        let mut position = 0;
        let p1: f64 = utils::read(stream, little_endian, &mut position);
        let p2: f64 = utils::read(stream, little_endian, &mut position);
        let p3: f64 = utils::read(stream, little_endian, &mut position);
        let p4: f64 = utils::read(stream, little_endian, &mut position);
        let p5: f64 = utils::read(stream, little_endian, &mut position);
        let p6: f64 = utils::read(stream, little_endian, &mut position);

        (
            ConversionPoly {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionExponetial {
    pub p1: f64,
    pub p2: f64,
    pub p3: f64,
    pub p4: f64,
    pub p5: f64,
    pub p6: f64,
    pub p7: f64,
}

impl ConversionExponetial {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionExponetial, usize) {
        let mut position = 0;
        let p1: f64 = utils::read(stream, little_endian, &mut position);
        let p2: f64 = utils::read(stream, little_endian, &mut position);
        let p3: f64 = utils::read(stream, little_endian, &mut position);
        let p4: f64 = utils::read(stream, little_endian, &mut position);
        let p5: f64 = utils::read(stream, little_endian, &mut position);
        let p6: f64 = utils::read(stream, little_endian, &mut position);
        let p7: f64 = utils::read(stream, little_endian, &mut position);

        (
            ConversionExponetial {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
                p7,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionLog {
    pub p1: f64,
    pub p2: f64,
    pub p3: f64,
    pub p4: f64,
    pub p5: f64,
    pub p6: f64,
    pub p7: f64,
}

impl ConversionLog {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionLog, usize) {
        let mut position = 0;
        let p1: f64 = utils::read(stream, little_endian, &mut position);
        let p2: f64 = utils::read(stream, little_endian, &mut position);
        let p3: f64 = utils::read(stream, little_endian, &mut position);
        let p4: f64 = utils::read(stream, little_endian, &mut position);
        let p5: f64 = utils::read(stream, little_endian, &mut position);
        let p6: f64 = utils::read(stream, little_endian, &mut position);
        let p7: f64 = utils::read(stream, little_endian, &mut position);

        (
            ConversionLog {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
                p7,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionRational {
    pub p1: f64,
    pub p2: f64,
    pub p3: f64,
    pub p4: f64,
    pub p5: f64,
    pub p6: f64,
}

impl ConversionRational {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionRational, usize) {
        let mut position = 0;
        let p1: f64 = utils::read(stream, little_endian, &mut position);
        let p2: f64 = utils::read(stream, little_endian, &mut position);
        let p3: f64 = utils::read(stream, little_endian, &mut position);
        let p4: f64 = utils::read(stream, little_endian, &mut position);
        let p5: f64 = utils::read(stream, little_endian, &mut position);
        let p6: f64 = utils::read(stream, little_endian, &mut position);

        (
            ConversionRational {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Table {
    #[allow(dead_code)]
    ConversionTabular,
}

#[derive(Debug, Clone)]
pub struct ConversionTabular {
    pub value: Vec<TableEntry>,
}

impl ConversionTabular {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionTabular, usize) {
        let mut position = 0;
        let mut value = Vec::new();
        for _i in 0..1 {
            let (temp, pos) = TableEntry::read(stream, little_endian);
            position += pos;
            value.push(temp);
        }

        (ConversionTabular { value }, position)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TableEntry {
    pub internal: f64,
    pub physical: f64,
}

impl TableEntry {
    #[allow(dead_code)]
    pub fn write() {}
    pub fn read(stream: &[u8], little_endian: bool) -> (TableEntry, usize) {
        let mut position = 0;
        let internal = utils::read(stream, little_endian, &mut position);
        let physical = utils::read(stream, little_endian, &mut position);

        (TableEntry { internal, physical }, position)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Text {
    #[allow(dead_code)]
    ConversionTextFormula,
    #[allow(dead_code)]
    ConversionTextRangeTable,
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionTextFormula {
    pub formula: [u8; 256],
}

impl ConversionTextFormula {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], _little_endian: bool) -> (ConversionTextFormula, usize) {
        let mut position = 0;
        let formula: [u8; 256] = stream.try_into().expect("msg");
        position += formula.len();

        (ConversionTextFormula { formula }, position)
    }
}

#[derive(Debug, Clone)]
pub struct ConversionTextTable {
    pub table: Vec<TextTableEntry>,
}

impl ConversionTextTable {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], little_endian: bool, number: usize) -> (ConversionTextTable, usize) {
        let mut position = 0;
        let mut table = Vec::new();
        for _i in 0..number - 1 {
            let (table_entry, pos) = TextTableEntry::read(stream, little_endian);
            table.push(table_entry);
            position += pos;
        }

        (ConversionTextTable { table }, position)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextTableEntry {
    pub internal: f64,
    pub text: [u8; 32],
}

impl TextTableEntry {
    #[allow(dead_code)]
    pub fn write() {}
    pub fn read(stream: &[u8], little_endian: bool) -> (TextTableEntry, usize) {
        let mut position = 0;
        let internal = utils::read(stream, little_endian, &mut position);
        let text: [u8; 32] = stream.try_into().expect("msg");

        (TextTableEntry { internal, text }, position)
    }
}

#[derive(Debug, Clone)]
pub struct ConversionTextRangeTable {
    pub undef1: f64,
    pub undef2: f64,
    pub txblock: u32,
    pub entry: Vec<TextRange>,
}

impl ConversionTextRangeTable {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], little_endian: bool) -> (ConversionTextRangeTable, usize) {
        let mut position = 0;
        let undef1 = utils::read(stream, little_endian, &mut position);
        let undef2 = utils::read(stream, little_endian, &mut position);
        let txblock = utils::read(stream, little_endian, &mut position);
        let entry = Vec::new();

        (
            ConversionTextRangeTable {
                undef1,
                undef2,
                txblock,
                entry,
            },
            position,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextRange {
    pub lower: f64,
    pub upper: f64,
    pub txblock: u32,
}

impl TextRange {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(stream: &[u8], little_endian: bool) -> (TextRange, usize) {
        let mut position = 0;
        let lower = utils::read(stream, little_endian, &mut position);
        let upper = utils::read(stream, little_endian, &mut position);
        let txblock = utils::read(stream, little_endian, &mut position);

        (
            TextRange {
                lower,
                upper,
                txblock,
            },
            position,
        )
    }
}
