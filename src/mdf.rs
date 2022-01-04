use crate::mdf3::{self, MDF3};
// use crate::mdf4;
use crate::signal::Signal;

pub struct MDF{
	filepath: String, 
	file: MDF3,
}


impl MDFFile for MDF {
	fn new(filepath: &str) -> Self{
		Self{
			filepath: filepath.to_string(), 
			file: MDF3::new(filepath)
		}
	}

    fn read_all(&mut self){
		self.file.read_all();
	}

    fn list(&mut self){
		self.file.list();
	}

    fn list_channels(&self){
		self.file.list_channels();
	}

    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Signal{
		self.file.read(datagroup, channel_grp, channel)
	}

    fn cut(&self, start: f64, end: f64, include_ends: bool, time_from_zero: bool){
		self.file.cut(start, end, include_ends, time_from_zero)
	}

    fn export(&self, format: &str, filename: &str){
		self.file.export(format, filename)
	}
    fn filter(&self, channels: &str){
		self.file.filter(channels)
	}
    fn resample(&self, raster: RasterType, version: &str, time_from_zero: bool) -> Self{
		Self {
			filepath: self.filepath.clone(),
			file: self.file.resample(raster, version, time_from_zero),
		}
		
	}
    fn select(
        &self,
        channels: ChannelsType,
        record_offset: isize,
        raw: bool,
        copy_master: bool,
        ignore_value2text_conversions: bool,
        record_count: isize,
        validate: bool,
    ) -> Vec<Signal>{
		self.file.select(channels, record_offset, raw, copy_master, ignore_value2text_conversions, record_count, validate)
	}
}


pub enum File {
	MDF3, 
	// v4: MDF4,
}


pub trait MDFFile {
    fn new(filepath: &str) -> Self;

    fn read_all(&mut self);

    fn list(&mut self);

    fn list_channels(&self);

    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Signal;

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
