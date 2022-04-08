use std::fs::File;
use std::io::Read;

use crate::mdf3::MDF3;
use crate::mdf4::MDF4;
use crate::record::Record;
use crate::signal::Signal;
use crate::utils;

enum MDFVersion {
    MDF3,
    MDF4,
}

enum MDFType {
    MDF3(MDF3),
    MDF4(MDF4),
}

impl MDFType {
    fn check_version(filepath: &str) -> MDFVersion {
        let file = File::open(filepath).expect("Could not read file");
        let mut id_stream = Vec::with_capacity(64);
        let _ = file.read_exact(&mut id_stream);

        let mut pos = 0;
        let little_endian = true;

        let id_file: [u8; 8] = utils::read(&id_stream, little_endian, &mut pos);
        let id_vers: [u8; 8] = utils::read(&id_stream, little_endian, &mut pos);
        let id_prog: [u8; 8] = utils::read(&id_stream, little_endian, &mut pos);
        let id_reserved1: [u8; 4] = utils::read(&id_stream, little_endian, &mut pos);
        let id_ver: u16 = utils::read(&id_stream, little_endian, &mut pos);
        let id_reserved2: [u8; 34] = utils::read(&id_stream, little_endian, &mut pos);

        if !utils::eq(&id_file, &[b'M', b'D', b'F', b' ', b' ', b' ', b' ', b' ']) {
            panic!("Error: Unknown file type");
        }

        let s = String::from_utf8_lossy(&id_vers).into_owned();
        let version = s.split('.');
        let major_version = version.next().unwrap().parse::<usize>().unwrap();
        let minor_version = version.next().unwrap().parse::<usize>().unwrap();

        let mdf_version = match major_version {
            3 => MDFVersion::MDF3,
            4 => MDFVersion::MDF4,
            _ => panic!("Unknown MDF file version"),
        };

        mdf_version
    }
}

impl MDFFile for MDFType {
    fn channels(&self) -> Vec<MdfChannel> {
        self.channels()
    }
    fn find_time_channel(
        &self,
        datagroup: usize,
        channel_grp: usize,
    ) -> Result<usize, &'static str> {
        self.find_time_channel(datagroup, channel_grp)
    }

    fn read_channel(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Vec<Record> {
        self.read_channel(datagroup, channel_grp, channel)
    }

    #[must_use]
    fn new(filepath: &str) -> Self {
        let version = MDFType::check_version(filepath);

        let file = match version {
            MDFVersion::MDF3 => MDFType::MDF3(MDF3::new(filepath)),
            MDFVersion::MDF4 => MDFType::MDF4(MDF4::new(filepath)),
        };
        //let file = MDFType::new(filepath);
        // Self {
        //     filepath: filepath.to_string(),
        //     channels: file.channels(),
        //     file,
        // }
        file
    }

    fn read_all(&mut self) {
        self.read_all();
    }

    fn list(&mut self) {
        self.list();
    }

    fn list_channels(&self) {
        self.list_channels();
    }

    #[must_use]
    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Signal {
        self.read(datagroup, channel_grp, channel)
    }

    fn cut(&self, start: f64, end: f64, include_ends: bool, time_from_zero: bool) {
        self.cut(start, end, include_ends, time_from_zero)
    }

    fn export(&self, format: &str, filename: &str) {
        self.export(format, filename)
    }

    fn filter(&self, channels: &str) {
        self.filter(channels)
    }

    #[must_use]
    fn resample(&self, raster: RasterType, version: &str, time_from_zero: bool) -> Self {
        // Self {
        //     filepath: self.filepath.clone(),
        self.resample(raster, version, time_from_zero)
        //     channels: Vec::new(),
        // }
    }
    // #[must_use]
    // fn select(
    //     &self,
    //     channels: ChannelsType,
    //     record_offset: isize,
    //     raw: bool,
    //     copy_master: bool,
    //     ignore_value2text_conversions: bool,
    //     record_count: isize,
    //     validate: bool,
    // ) -> Vec<Signal>;
}

pub(crate) struct MDF {
    pub filepath: String,
    pub file: MDFType,
    pub channels: Vec<MdfChannel>,
}

impl MDF {
    pub(crate) fn search_channels(&self, channel_name: &str) -> Result<MdfChannel, &'static str> {
        let mut channels_match = Vec::with_capacity(self.channels.len());

        for channel in &self.channels {
            if channel.name.eq(&channel_name) {
                channels_match.push(channel.clone());
            }
        }

        match channels_match.len() {
            0 => Err("Channel not found"),
            1 => Ok(channels_match[0].clone()),
            l if 1 < l => Err("Multiple matches found"),
            _ => Err(r#"Unknown error measuring length of matching channels"#),
        }
    }

    pub(crate) fn list_channels(&self) {
        for channel in &self.channels {
            println!(
                "Channel: {}, DG: {}, CG: {}, CN: {}",
                channel.name, channel.data_group, channel.channel_group, channel.channel
            );
        }
    }

    pub(crate) fn read_channel(self, channel: MdfChannel) -> Signal {
        self.file.read(
            channel.data_group as usize,
            channel.channel_group as usize,
            channel.channel as usize,
        )
    }
}

impl MDFFile for MDF {
    fn channels(&self) -> Vec<MdfChannel> {
        self.file.channels()
    }

    fn find_time_channel(
        &self,
        datagroup: usize,
        channel_grp: usize,
    ) -> Result<usize, &'static str> {
        self.file.find_time_channel(datagroup, channel_grp)
    }

    fn read_channel(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Vec<Record> {
        self.file.read_channel(datagroup, channel_grp, channel)
    }

    fn new(filepath: &str) -> Self {
        let file = MDFType::new(filepath);
        Self {
            filepath: filepath.to_string(),
            channels: file.channels(),
            file,
        }
    }

    fn read_all(&mut self) {
        self.file.read_all();
    }

    fn list(&mut self) {
        self.file.list();
    }

    fn list_channels(&self) {
        self.file.list_channels();
    }

    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Signal {
        self.file.read(datagroup, channel_grp, channel)
    }

    fn cut(&self, start: f64, end: f64, include_ends: bool, time_from_zero: bool) {
        self.file.cut(start, end, include_ends, time_from_zero)
    }

    fn export(&self, format: &str, filename: &str) {
        self.file.export(format, filename)
    }
    fn filter(&self, channels: &str) {
        self.file.filter(channels)
    }
    fn resample(&self, raster: RasterType, version: &str, time_from_zero: bool) -> Self {
        Self {
            filepath: self.filepath.clone(),
            file: self.file.resample(raster, version, time_from_zero),
            channels: Vec::new(),
        }
    }
    // fn select(
    //     &self,
    //     channels: ChannelsType,
    //     record_offset: isize,
    //     raw: bool,
    //     copy_master: bool,
    //     ignore_value2text_conversions: bool,
    //     record_count: isize,
    //     validate: bool,
    // ) -> Vec<Signal> {
    //     self.file.select(
    //         channels,
    //         record_offset,
    //         raw,
    //         copy_master,
    //         ignore_value2text_conversions,
    //         record_count,
    //         validate,
    //     )
    // }
}

pub(crate) trait MDFFile {
    fn channels(&self) -> Vec<MdfChannel>;
    fn find_time_channel(
        &self,
        _datagroup: usize,
        channel_grp: usize,
    ) -> Result<usize, &'static str>;

    fn read_channel(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Vec<Record>;

    #[must_use]
    fn new(filepath: &str) -> Self;

    fn read_all(&mut self);

    fn list(&mut self);

    fn list_channels(&self);

    #[must_use]
    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Signal;

    fn cut(&self, start: f64, end: f64, include_ends: bool, time_from_zero: bool);

    fn export(&self, format: &str, filename: &str);
    fn filter(&self, channels: &str);
    #[must_use]
    fn resample(&self, raster: RasterType, version: &str, time_from_zero: bool) -> Self;
    // #[must_use]
    // fn select(
    //     &self,
    //     channels: ChannelsType,
    //     record_offset: isize,
    //     raw: bool,
    //     copy_master: bool,
    //     ignore_value2text_conversions: bool,
    //     record_count: isize,
    //     validate: bool,
    // ) -> Vec<Signal>;
}

pub(crate) struct RasterType {}

pub(crate) struct ChannelsType {}

pub(crate) struct TimeChannel {
    pub time: Vec<f64>,
    pub data: Vec<f64>,
}

impl TimeChannel {
    pub(crate) fn new(times: Vec<Record>, datas: Vec<Record>) -> Self {
        let mut t = Vec::with_capacity(times.len());
        let mut d = Vec::with_capacity(datas.len());

        for time in times {
            t.push(time.extract());
        }

        for data in datas {
            d.push(data.extract());
        }

        Self { time: t, data: d }
    }

    pub(crate) fn max_time(&self) -> f64 {
        return *self.time.last().expect("Error reading time");
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MdfChannel {
    pub name: String,
    pub data_group: u64,
    pub channel: u64,
    pub channel_group: u64,
}
