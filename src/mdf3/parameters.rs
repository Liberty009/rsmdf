#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Parameters {
    #[allow(dead_code)]
    Linear,
    #[allow(dead_code)]
    Poly,
    #[allow(dead_code)]
    Exponetial,
    #[allow(dead_code)]
    Log,
    #[allow(dead_code)]
    Rational,
}

impl Parameters {
    #[allow(dead_code)]
    pub fn write() {}
    #[allow(dead_code)]
    pub fn read(_data: &[u8], _little_endian: bool) -> (Parameters, usize) {
        (Parameters::Linear, 10)
    }
}
