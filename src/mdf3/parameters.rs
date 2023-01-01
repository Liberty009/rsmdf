#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parameters {
    Linear,
    Poly,
    Exponetial,
    Log,
    Rational,
}

impl Parameters {
    pub fn write() {}

    pub fn read(_data: &[u8], _little_endian: bool) -> (Parameters, usize) {
        (Parameters::Linear, 10)
    }
}
