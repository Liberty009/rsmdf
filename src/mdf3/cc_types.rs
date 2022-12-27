use crate::utils;
use chrono::{NaiveDateTime, TimeZone};
use itertools::izip;

use super::tx_block::Txblock;

pub enum ConversionTypes {
    Linear(Linear),
    TabularInterpolation(TabularInterpolation),
    Tabular(Tabular),
    Polynomial(Polynomial),
    Exponential(Exponential),
    Logarithmic(Logarithmic),
    Rational(Rational),
    TextFormula(TextFormula),
    TextTable(TextTable),           //(COMPU_VTAB)
    TextRangeTable(TextRangeTable), // (COMPU_VTAB_RANGE)
    Date(Date),                     // (Based on 7 Byte Date data structure)
    Time(Time),                     // (Based on 6 Byte Time data structure)
    Direct(Direct),                 //1:1 conversion formula (Int = Phys)
}

impl ConversionTypes {
    #[allow(dead_code)]
    fn new(stream: &[u8], position: usize, little_endian: bool, conversion: u16) -> Self {
        match conversion {
            0 => {
                let (_pos, con) = Linear::read(stream, position, little_endian);
                Self::Linear(con)
            }
            1 => {
                let (_pos, con) = TabularInterpolation::read(stream, position, little_endian);
                Self::TabularInterpolation(con)
            }
            2 => {
                let (_pos, con) = Tabular::read(stream, position, little_endian);
                Self::Tabular(con)
            }
            6 => {
                let (_pos, con) = Polynomial::read(stream, position, little_endian);
                Self::Polynomial(con)
            }
            7 => {
                let (_pos, con) = Exponential::read(stream, position, little_endian);
                Self::Exponential(con)
            }
            8 => {
                let (_pos, con) = Logarithmic::read(stream, position, little_endian);
                Self::Logarithmic(con)
            }
            9 => {
                let (_pos, con) = Rational::read(stream, position, little_endian);
                Self::Rational(con)
            }
            10 => {
                let (_pos, con) = TextFormula::read(stream, position, little_endian);
                Self::TextFormula(con)
            }
            11 => {
                let (_pos, con) = TextTable::read(stream, position, little_endian);
                Self::TextTable(con)
            } //(COMPU_VTAB)
            12 => {
                let (_pos, con) = TextRangeTable::read(stream, position, little_endian);
                Self::TextRangeTable(con)
            } // (COMPU_VTAB_RANGE)
            132 => {
                let (_pos, con) = Date::read(stream, position, little_endian);
                Self::Date(con)
            } // (Based on 7 Byte Date data structure)
            133 => {
                let (_pos, con) = Time::read(stream, position, little_endian);
                Self::Time(con)
            } // (Based on 6 Byte Time data structure)
            65535 => {
                let (_pos, con) = Direct::read(stream, position, little_endian);
                Self::Direct(con)
            } //1:1 conversion formula (Int = Phys)
            _ => panic!("Error: Unknown conversion type"),
        }
    }

    #[allow(dead_code)]
    fn convert(&self, data: &[f64]) -> Physical {
        match self {
            Self::Linear(conv) => conv.convert(data),
            Self::TabularInterpolation(conv) => conv.convert(data),
            Self::Tabular(conv) => conv.convert(data),
            Self::Polynomial(conv) => conv.convert(data),
            Self::Exponential(conv) => conv.convert(data),
            Self::Logarithmic(conv) => conv.convert(data),
            Self::Rational(conv) => conv.convert(data),
            Self::TextFormula(conv) => conv.convert(data),
            Self::TextTable(conv) => conv.convert(data),
            Self::TextRangeTable(conv) => conv.convert(data),
            Self::Date(conv) => conv.convert(data),
            Self::Time(conv) => conv.convert(data),
            Self::Direct(conv) => conv.convert(data),
        }
    }
}

pub enum Physical {
    Text(Vec<String>),
    Value(Vec<f64>),
}

trait Conversion {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self);
    fn convert(&self, data: &[f64]) -> Physical;
}

pub struct Linear {
    p1: f64,
    p2: f64,
}
impl Conversion for Linear {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let p1 = utils::read(stream, little_endian, &mut pos);
        let p2 = utils::read(stream, little_endian, &mut pos);
        (pos, Self { p1, p2 })
    }

    fn convert(&self, data: &[f64]) -> Physical {
        let mut physical = Vec::with_capacity(data.len());
        for (int, phys) in izip!(data, &mut physical) {
            *phys = int * self.p2 + self.p1;
        }

        Physical::Value(physical)
    }
}

pub struct TabularInterpolation {}
impl Conversion for TabularInterpolation {
    fn read(_stream: &[u8], _position: usize, _little_endian: bool) -> (usize, Self) {
        todo!()
    }

    fn convert(&self, _data: &[f64]) -> Physical {
        todo!()
    }
}

pub struct Tabular {}
impl Conversion for Tabular {
    fn read(_stream: &[u8], _position: usize, _little_endian: bool) -> (usize, Self) {
        todo!()
    }

    fn convert(&self, _data: &[f64]) -> Physical {
        todo!()
    }
}

pub struct Polynomial {
    p1: f64,
    p2: f64,
    p3: f64,
    p4: f64,
    p5: f64,
    p6: f64,
}

impl Conversion for Polynomial {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let p1 = utils::read(stream, little_endian, &mut pos);
        let p2 = utils::read(stream, little_endian, &mut pos);
        let p3 = utils::read(stream, little_endian, &mut pos);
        let p4 = utils::read(stream, little_endian, &mut pos);
        let p5 = utils::read(stream, little_endian, &mut pos);
        let p6 = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
            },
        )
    }

    fn convert(&self, data: &[f64]) -> Physical {
        let mut physical = Vec::with_capacity(data.len());
        for (int, phys) in izip!(data, &mut physical) {
            let subtracted = int - self.p5 - self.p6;
            *phys = (self.p2 - (self.p4 * subtracted)) / (self.p3 * subtracted - self.p1);
        }

        Physical::Value(physical)
    }
}

pub struct Exponential {
    p1: f64,
    p2: f64,
    p3: f64,
    p4: f64,
    p5: f64,
    p6: f64,
    p7: f64,
}

impl Conversion for Exponential {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let p1 = utils::read(stream, little_endian, &mut pos);
        let p2 = utils::read(stream, little_endian, &mut pos);
        let p3 = utils::read(stream, little_endian, &mut pos);
        let p4 = utils::read(stream, little_endian, &mut pos);
        let p5 = utils::read(stream, little_endian, &mut pos);
        let p6 = utils::read(stream, little_endian, &mut pos);
        let p7 = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
                p7,
            },
        )
    }

    fn convert(&self, data: &[f64]) -> Physical {
        let mut physical = Vec::with_capacity(data.len());

        if self.p4 == 0.0 {
            for (int, phys) in izip!(data, &mut physical) {
                let step1 = (int - self.p7) * self.p6;
                let step2 = (step1 - self.p3) / self.p1;
                *phys = (step2.ln()) / self.p2;
            }
        } else {
            // if p1 == 0
            for (int, phys) in izip!(data, &mut physical) {
                let step1 = (self.p3) / (int - self.p7);
                let step2 = (step1 - self.p6) / self.p4;
                *phys = (step2.ln()) / self.p5;
            }
        }

        Physical::Value(physical)
    }
}

pub struct Logarithmic {
    p1: f64,
    p2: f64,
    p3: f64,
    p4: f64,
    p5: f64,
    p6: f64,
    p7: f64,
}
impl Conversion for Logarithmic {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let p1 = utils::read(stream, little_endian, &mut pos);
        let p2 = utils::read(stream, little_endian, &mut pos);
        let p3 = utils::read(stream, little_endian, &mut pos);
        let p4 = utils::read(stream, little_endian, &mut pos);
        let p5 = utils::read(stream, little_endian, &mut pos);
        let p6 = utils::read(stream, little_endian, &mut pos);
        let p7 = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
                p7,
            },
        )
    }

    fn convert(&self, data: &[f64]) -> Physical {
        let mut physical = Vec::with_capacity(data.len());
        if self.p4 == 0.0 {
            for (int, phys) in izip!(data, &mut physical) {
                let step1 = int - self.p7;
                let step2 = (step1 * self.p6 - self.p3) / self.p1;
                *phys = (step2.exp()) / self.p2;
            }
        } else {
            //self.p1 == 0.0
            for (int, phys) in izip!(data, &mut physical) {
                let step1 = (self.p3) / (int - self.p7);
                let step2 = (step1 - self.p6) / self.p4;
                *phys = (step2.exp()) / self.p5;
            }
        }

        Physical::Value(physical)
    }
}

pub struct Rational {
    p1: f64,
    p2: f64,
    p3: f64,
    p4: f64,
    p5: f64,
    p6: f64,
}

impl Conversion for Rational {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let p1: f64 = utils::read(stream, little_endian, &mut pos);
        let p2: f64 = utils::read(stream, little_endian, &mut pos);
        let p3: f64 = utils::read(stream, little_endian, &mut pos);
        let p4: f64 = utils::read(stream, little_endian, &mut pos);
        let p5: f64 = utils::read(stream, little_endian, &mut pos);
        let p6: f64 = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                p1,
                p2,
                p3,
                p4,
                p5,
                p6,
            },
        )
    }

    fn convert(&self, data: &[f64]) -> Physical {
        let mut physical = Vec::with_capacity(data.len());

        for (int, phys) in izip!(data, &mut physical) {
            *phys = (self.p1 * int.powi(2) + self.p2 * int + self.p3)
                / (self.p4 * int.powi(2) + self.p5 * int + self.p6);
        }

        Physical::Value(physical)
    }
}

pub struct TextFormula {
    #[allow(dead_code)]
    formula: String,
}

impl Conversion for TextFormula {
    fn read(_stream: &[u8], _position: usize, _little_endian: bool) -> (usize, Self) {
        todo!()
    }

    fn convert(&self, _data: &[f64]) -> Physical {
        todo!()
    }
}

trait LookupTable {
    fn lookup(&self, search: &[f64]) -> Physical;
}

pub struct TextTable {
    table: Vec<TextTableEntry>,
}

impl TextTable {
    fn find(&self, search: f64) -> String {
        let entry = &self.table.iter().find(|x| x.internal == search);
        match entry {
            Some(text) => text.text.clone(),
            None => {
                let last = self.table.last().expect("Text table is empty");
                last.text.clone()
            }
        }
    }
}

impl Conversion for TextTable {
    fn read(_stream: &[u8], _position: usize, _little_endian: bool) -> (usize, Self) {
        todo!()
    }

    fn convert(&self, data: &[f64]) -> Physical {
        let mut physical = Vec::with_capacity(data.len());
        for (term, phys) in izip!(data, &mut physical) {
            *phys = self.find(*term);
        }

        Physical::Text(physical)
    }
}

pub struct TextTableEntry {
    internal: f64,
    text: String,
}

pub struct TextRangeTable {
    table: Vec<TextRangeEntry>,
    default: String,
}

impl TextRangeTable {
    fn find(&self, term: f64) -> String {
        let mut result = String::new();
        for entry in &self.table {
            if (entry.lower <= term) && (term < entry.upper) {
                result = entry.text.clone();
                break;
            }
        }

        if result.is_empty() {
            result = self.default.clone();
        }

        result
    }
}

impl Conversion for TextRangeTable {
    fn read(_stream: &[u8], _position: usize, _little_endian: bool) -> (usize, Self) {
        todo!()
    }

    fn convert(&self, data: &[f64]) -> Physical {
        let mut physical = Vec::with_capacity(data.len());
        for (term, phys) in izip!(data, &mut physical) {
            *phys = self.find(*term);
        }

        Physical::Text(physical)
    }
}

#[derive(Debug, Clone)]
pub struct TextRangeEntry {
    lower: f64,
    upper: f64,
    text: String,
}

use crate::mdf3::mdf3_block::Mdf3Block;

impl TextRangeEntry {
    #[allow(dead_code)]
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let lower = utils::read(stream, little_endian, &mut pos);
        let upper = utils::read(stream, little_endian, &mut pos);
        let txblock_pos: u32 = utils::read(stream, little_endian, &mut pos);
        let (_pos, tx) = Txblock::read(stream, txblock_pos as usize, little_endian);
        let text = tx.name();

        (pos, Self { lower, upper, text })
    }
}

pub struct Date {
    #[allow(dead_code)]
    ms: u16,
    #[allow(dead_code)]
    min: u8,
    #[allow(dead_code)]
    hour: u8,
    #[allow(dead_code)]
    day: u8,
    #[allow(dead_code)]
    month: u8,
    #[allow(dead_code)]
    year: u8,
}

impl Date {
    #[allow(dead_code)]
    fn to_datetime(&self) -> NaiveDateTime {
        let _date_time = chrono::Local.with_ymd_and_hms(
            self.year as i32,
            self.month as u32,
            self.day as u32,
            self.hour as u32,
            self.min as u32,
            (self.ms / 1000) as u32,
            // (self.ms % 1000) as u32,
        );
        todo!()
    }
}

impl Conversion for Date {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let ms = utils::read(stream, little_endian, &mut pos);
        let min = utils::read(stream, little_endian, &mut pos);
        let hour = utils::read(stream, little_endian, &mut pos);
        let day = utils::read(stream, little_endian, &mut pos);
        let month = utils::read(stream, little_endian, &mut pos);
        let year = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                ms,
                min,
                hour,
                day,
                month,
                year,
            },
        )
    }

    fn convert(&self, _data: &[f64]) -> Physical {
        todo!()
    }
}

pub struct Time {
    #[allow(dead_code)]
    ms: u32,
    #[allow(dead_code)]
    days: u8,
}

impl Conversion for Time {
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let ms = utils::read(stream, little_endian, &mut pos);
        let days = utils::read(stream, little_endian, &mut pos);

        (pos, Self { ms, days })
    }

    fn convert(&self, _data: &[f64]) -> Physical {
        todo!()
    }
}

pub struct Direct {}
impl Conversion for Direct {
    fn read(_stream: &[u8], _position: usize, _little_endian: bool) -> (usize, Self) {
        todo!()
    }

    fn convert(&self, data: &[f64]) -> Physical {
        Physical::Value(data.to_vec())
    }
}
