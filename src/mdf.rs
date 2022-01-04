use crate::mdf3::{self, MDF3};
// use crate::mdf4;
use crate::signal::Signal;

pub struct MDF{
	filepath: String, 
	file: MDF3,
	channels: Vec<MdfChannel>,
}

impl MDF {
	pub fn search_channels(self, channel_name: &str) -> Result<MdfChannel, &'static str>{
		
		let mut channels_match = Vec::new();

		for channel in self.channels {
			if channel.name.contains(&channel_name) {
				channels_match.push(channel.clone());
			}
		}

		if channels_match.len() == 1 {
			return Ok(channels_match[0].clone());
		} else if 1 < channels_match.len() {
			return Err("Multiple matches found");
		}
		
		
		Err("Channel not found")
	}
}


impl MDFFile for MDF {
	fn new(filepath: &str) -> Self{
		Self{
			filepath: filepath.to_string(), 
			file: MDF3::new(filepath), 
			channels: Vec::new(),
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
			channels: Vec::new(),
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

#[derive(Debug, Clone)]
pub struct MdfChannel{
	pub name: String, 
	pub data_group: u64, 
	pub channel: u64, 
	pub channel_group: u64,
}

