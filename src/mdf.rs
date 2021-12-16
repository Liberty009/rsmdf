use crate::mdf3;
use crate::signal::Signal;

pub trait MDF {
    fn new(filepath: &str) -> Self;

    fn read_all(&mut self);

    fn list(&mut self);

    fn list_channels(&self);

    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> TimeChannel;

    fn cut(&self, start: f64, end: f64, include_ends: bool, time_from_zero: bool);

    fn export(&self, format: &str, filename: &str);
    fn filter(&self, channels: &str);
    fn resample(&self, raster: RasterType, version: &str, time_from_zero: bool) -> Self;
    fn select(
        &self,
        channels: ChannelsType,
        record_offset: isize,
        raw: bool,
        copy_master: bool,
        ignore_value2text_conversions: bool,
        record_count: isize,
        validate: bool,
    ) -> Vec<Signal>;
}

pub struct RasterType {}

pub struct ChannelsType {}

pub struct TimeChannel {
    pub time: Vec<f64>,
    pub data: Vec<f64>,
}

impl TimeChannel {
    pub fn new(times: Vec<mdf3::Record>, datas: Vec<mdf3::Record>) -> Self {
        let mut t = Vec::new();
        let mut d = Vec::new();

        for time in times {
            t.push(time.extract());
        }

        for data in datas {
            d.push(data.extract());
        }

        Self { time: t, data: d }
    }

    pub fn max_time(&self) -> f64 {
        return *self.time.last().expect("Error reading time");
    }
}

// pub trait IDBLOCK {
// 	fn read(stream: &[u8]) ->  (usize, bool, (dyn IDBLOCK + 'static), );
// }
// pub trait DGBLOCK {
// 	fn read(stream: &[u8]) -> Self;
// }
// pub trait CNBLOCK {}
// pub trait CGBLOCK {}
