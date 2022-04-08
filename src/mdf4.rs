use crate::mdf::{self, MdfChannel, RasterType};
use crate::signal::{self, Signal};
use crate::utils;
use std::io::prelude::*;
use std::{convert::TryInto, fs::File};

struct BlockHeader {
    id: [u8; 4],
    reserved0: [u8; 4],
    length: u64,
    link_count: u64,
}

impl Block for BlockHeader {
    fn new() -> Self {
        Self {
            id: [0; 4],
            reserved0: [0; 4],
            length: 0,
            link_count: 0,
        }
    }
    fn default() -> Self {
        Self {
            id: [0; 4],
            reserved0: [0; 4],
            length: 0,
            link_count: 0,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let id: [u8; 4] = stream[pos..pos + 4].try_into().expect("msg");
        pos += id.len();
        let reserved0: [u8; 4] = stream[pos..pos + 4].try_into().expect("msg");
        pos += reserved0.len();

        let length = utils::read(stream, little_endian, &mut pos);
        let link_count = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                id,
                reserved0,
                length,
                link_count,
            },
        )
    }
}

fn link_extract(
    stream: &[u8],
    position: usize,
    little_endian: bool,
    no_links: u64,
) -> (usize, Vec<u64>) {
    let mut links = Vec::new();
    let mut pos = position;

    for i in 0..no_links {
        let address: u64 = utils::read(stream, little_endian, &mut pos);
        links.push(address);
    }

    (pos, links)
}

trait Block {
    fn new() -> Self;
    fn default() -> Self;
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self);
}

#[derive(Debug, Clone)]
pub(crate) struct MDF4 {
    #[allow(dead_code)]
    id: IDBLOCK,
    #[allow(dead_code)]
    header: HDBLOCK,
    #[allow(dead_code)]
    comment: TXBLOCK,
    data_groups: Vec<DGBLOCK>,
    channels: Vec<CNBLOCK>,
    channel_groups: Vec<CGBLOCK>,
    little_endian: bool,
    file: Vec<u8>,
}

impl mdf::MDFFile for MDF4 {
    fn channels(&self) -> Vec<MdfChannel> {
        let mut channels = Vec::new();

        let mut dg = Vec::new();
        let mut cg = Vec::new();
        let mut ch = Vec::new();

        let little_endian = true;

        let (position, id_block) = IDBLOCK::read(&self.file, 0, little_endian);
        let (hd_block, _pos) = HDBLOCK::read(&self.file, position, little_endian);

        let mut next_dg = hd_block.data_group_block;

        while next_dg != 0 {
            let dg_block = DGBLOCK::read(&self.file, &mut (next_dg as usize), little_endian);
            next_dg = dg_block.next;
            let mut next_cg = dg_block.first;

            dg.push(dg_block);

            while next_cg != 0 {
                let (cg_block, _position) =
                    CGBLOCK::read(&self.file, next_cg as usize, little_endian);
                next_cg = cg_block.next;
                let mut next_cn = cg_block.first;
                cg.push(cg_block);

                println!("Channel Group: {}", cg_block.comment);

                while next_cn != 0 {
                    let (cn_block, _position) =
                        CNBLOCK::read(&self.file, next_cn as usize, little_endian);
                    next_cn = cn_block.next;
                    ch.push(cn_block);

                    let name = cn_block.name(&self.file, little_endian);
                    channels.push(mdf::MdfChannel {
                        name,
                        data_group: (dg.len() - 1) as u64,
                        channel_group: (cg.len() - 1) as u64,
                        channel: (ch.len() - 1) as u64,
                    });
                }
            }
        }

        channels
    }
    fn find_time_channel(
        &self,
        _datagroup: usize,
        channel_grp: usize,
    ) -> Result<usize, &'static str> {
        let channel_group =
            self.channel_groups[channel_grp].channels(&self.file, self.little_endian);
        for (i, channel) in channel_group.iter().enumerate() {
            if channel.channel_type == TIME_CHANNEL_TYPE {
                return Ok(i);
            }
        }

        Err("No time series found for the channel selected")
    }

    fn read_channel(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Vec<Record> {
        let channels: Vec<CNBLOCK> = self.channel_groups[channel_grp].channels(&self.file, true);
        let data_length = (self.channel_groups[channel_grp].record_number
            * self.channel_groups[channel_grp].record_size as u32)
            as usize;
        let data = &self.file[self.data_groups[datagroup].data_block as usize
            ..(self.data_groups[datagroup].data_block as usize + data_length)];

        println!(
            "Record Number: {}",
            self.channel_groups[channel_grp].record_number
        );

        let mut data_blocks: Vec<&[u8]> =
            vec![&[0]; self.channel_groups[channel_grp].record_number as usize];
        // let mut data_blocks = Vec::with_capacity(self.channel_groups[channel_grp].record_number as usize);
        println!("Vec len: {}", data_blocks.len());

        for (i, db) in data_blocks.iter_mut().enumerate() {
            *db = &data[(i * self.channel_groups[channel_grp].record_size as usize) as usize
                ..((i + 1) * self.channel_groups[channel_grp].record_size as usize) as usize];
        }
        // for i in 0..self.channel_groups[channel_grp].record_number {
        //     data_blocks.push(
        //         &data[(i * self.channel_groups[channel_grp].record_size as u32) as usize
        //             ..((i + 1) * self.channel_groups[channel_grp].record_size as u32) as usize],
        //     );
        // }

        let byte_offset = (self.channels[channel].start_offset / 8) as usize;
        let _bit_offset = self.channels[channel].start_offset % 8;

        let mut records =
            Vec::with_capacity(self.channel_groups[channel_grp].record_number as usize);
        let mut pos = 0_usize;
        for _i in 0..self.channel_groups[channel_grp].record_number {
            records.push(&data[pos..pos + self.channel_groups[channel_grp].record_size as usize]);
            pos += self.channel_groups[channel_grp].record_size as usize;
        }

        let mut raw_data =
            Vec::with_capacity(self.channel_groups[channel_grp].record_number as usize);
        let end = byte_offset + channels[channel].data_type.len();
        for rec in &records {
            raw_data.push(&rec[byte_offset..end])
        }

        let mut extracted_data =
            Vec::with_capacity(self.channel_groups[channel_grp].record_number as usize);
        for raw in raw_data {
            extracted_data.push(Record::new(raw, channels[channel].data_type));
        }

        extracted_data
    }

    #[must_use]
    fn new(filepath: &str) -> Self {
        let mut file = File::open(filepath).expect("Could not read file");
        let mut stream = Vec::new();
        let _ = file.read_to_end(&mut stream);
        let (id, pos, little_endian) = IDBLOCK::read(&stream);
        let (header, _pos) = HDBLOCK::read(&stream, pos, little_endian);
        let (comment, _pos) = TXBLOCK::read(&stream, header.file_comment as usize, little_endian);
        let mut mdf = Self {
            id,
            header,
            comment,
            data_groups: DGBLOCK::read_all(
                &stream,
                little_endian,
                header.data_group_block as usize,
            ),
            channels: Vec::new(),
            channel_groups: Vec::new(),
            little_endian,
            file: stream,
        };

        mdf.read_all();

        mdf
    }

    fn read_all(&mut self) {
        let mut channel_groups = Vec::with_capacity(self.data_groups.len());
        for group in &self.data_groups {
            channel_groups.append(&mut group.read_channel_groups(&self.file, self.little_endian));
        }

        let mut channels = Vec::new();
        for grp in &channel_groups {
            channels.append(&mut grp.channels(&self.file, self.little_endian));
        }

        self.channel_groups = channel_groups;
        self.channels = channels;
    }

    fn list(&mut self) {
        let (_id_block, position, little_endian) = IDBLOCK::read(&self.file);
        let (hd_block, _pos) = HDBLOCK::read(&self.file, position, little_endian);
        //position += pos;

        let dg = DGBLOCK::read_all(
            &self.file,
            little_endian,
            hd_block.data_group_block as usize,
        );
        self.data_groups = dg;
    }

    fn list_channels(&self) {
        let mut dg = Vec::new();
        let mut cg = Vec::new();
        let mut ch = Vec::new();

        let (_id_block, position, little_endian) = IDBLOCK::read(&self.file);
        let (hd_block, _pos) = HDBLOCK::read(&self.file, position, little_endian);
        //position += pos;

        let mut next_dg = hd_block.data_group_block;

        while next_dg != 0 {
            let dg_block = DGBLOCK::read(&self.file, little_endian, &mut (next_dg as usize));
            next_dg = dg_block.next;
            let mut next_cg = dg_block.first;

            dg.push(dg_block);

            while next_cg != 0 {
                let (cg_block, _position) =
                    CGBLOCK::read(&self.file, little_endian, next_cg as usize);
                next_cg = cg_block.next;
                let mut next_cn = cg_block.first;
                cg.push(cg_block);

                println!("Channel Group: {}", cg_block.comment);

                while next_cn != 0 {
                    let (cn_block, _position) =
                        CNBLOCK::read(&self.file, little_endian, next_cn as usize);
                    next_cn = cn_block.next;

                    ch.push(cn_block);
                }
            }
        }

        // (ch, cg, dg);
    }

    #[must_use]
    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Signal {
        let time_channel = self.find_time_channel(datagroup, channel_grp);
        let time_channel = match time_channel {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };
        println!("Time Channel: {}", time_channel);
        let time = self.read_channel(datagroup, channel_grp, time_channel);
        let some = self.read_channel(datagroup, channel_grp, channel);

        signal::Signal::new(
            time.iter().map(|x| x.extract()).collect(),
            some.iter().map(|x| x.extract()).collect(),
            "Unit".to_string(),
            "Measurement".to_string(),
            "This is some measurement".to_string(),
            false,
        )
    }

    fn cut(&self, start: f64, end: f64, include_ends: bool, time_from_zero: bool) {
        let _delta = if time_from_zero { start } else { 0.0 };
    }

    fn export(&self, format: &str, filename: &str) {}
    fn filter(&self, channels: &str) {}
    #[must_use]
    fn resample(&self, raster: RasterType, version: &str, time_from_zero: bool) -> Self {
        self.clone()
    }
}

#[derive(Debug, Clone)]
struct IDBLOCK {
    id_file: [u8; 8],
    id_vers: [u8; 8],
    id_prog: [u8; 8],
    id_reserved1: [u8; 4],
    id_ver: u16,
    id_reserved2: [u8; 34],
}
impl Block for IDBLOCK {
    fn new() -> Self {
        Self {
            id_file: [0; 8],
            id_vers: [0; 8],
            id_prog: [0; 8],
            id_reserved1: [0; 4],
            id_ver: 0,
            id_reserved2: [0; 34],
        }
    }
    fn default() -> Self {
        Self {
            id_file: [0; 8],
            id_vers: [0; 8],
            id_prog: [0; 8],
            id_reserved1: [0; 4],
            id_ver: 0,
            id_reserved2: [0; 34],
        }
    }
    fn read(stream: &[u8], _position: usize, _little_endian: bool) -> (usize, Self) {
        let mut pos = 0;
        let litte_endian = true;
        let id_file = utils::read(stream, _little_endian, &mut pos);
        let id_vers = utils::read(stream, litte_endian, &mut pos);
        let id_prog = utils::read(stream, litte_endian, &mut pos);
        let id_reserved1 = utils::read(stream, litte_endian, &mut pos);
        let id_ver = utils::read(stream, litte_endian, &mut pos);
        let id_reserved2 = utils::read(stream, litte_endian, &mut pos);

        (
            pos,
            Self {
                id_file,
                id_vers,
                id_prog,
                id_reserved1,
                id_ver,
                id_reserved2,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct HDBLOCK {
    hd_dg_first: u64,
    hd_fh_first: u64,
    hd_ch_first: u64,
    hd_at_first: u64,
    hd_ev_first: u64,
    hd_md_comment: u64,
    hd_start_time_ns: u64,
    hd_tz_offset_min: i16,
    hd_dst_offset_min: i16,
    hd_time_flags: u8,
    hd_time_class: u8,
    hd_flags: u8,
    hd_reserved: u8,
    hd_start_angle_rad: f64,
    hd_start_distance_m: f64,
}
impl Block for HDBLOCK {
    fn new() -> Self {
        HDBLOCK {
            hd_dg_first: 0,
            hd_fh_first: 0,
            hd_ch_first: 0,
            hd_at_first: 0,
            hd_ev_first: 0,
            hd_md_comment: 0,
            hd_start_time_ns: 0,
            hd_tz_offset_min: 0,
            hd_dst_offset_min: 0,
            hd_time_flags: 0,
            hd_time_class: 0,
            hd_flags: 0,
            hd_reserved: 0,
            hd_start_angle_rad: 0.0,
            hd_start_distance_m: 0.0,
        }
    }
    fn default() -> Self {
        HDBLOCK {
            hd_dg_first: 0,
            hd_fh_first: 0,
            hd_ch_first: 0,
            hd_at_first: 0,
            hd_ev_first: 0,
            hd_md_comment: 0,
            hd_start_time_ns: 0,
            hd_tz_offset_min: 0,
            hd_dst_offset_min: 0,
            hd_time_flags: 0,
            hd_time_class: 0,
            hd_flags: 0,
            hd_reserved: 0,
            hd_start_angle_rad: 0.0,
            hd_start_distance_m: 0.0,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);
        let (mut pos, address) = link_extract(stream, pos, little_endian, header.link_count);

        let hd_dg_first = address.remove(0);
        let hd_fh_first = address.remove(0);
        let hd_ch_first = address.remove(0);
        let hd_at_first = address.remove(0);
        let hd_ev_first = address.remove(0);
        let hd_md_comment = address.remove(0);

        let hd_start_time_ns = utils::read(stream, little_endian, &mut pos);
        let hd_tz_offset_min = utils::read(stream, little_endian, &mut pos);
        let hd_dst_offset_min = utils::read(stream, little_endian, &mut pos);
        let hd_time_flags = utils::read(stream, little_endian, &mut pos);
        let hd_time_class = utils::read(stream, little_endian, &mut pos);
        let hd_flags = utils::read(stream, little_endian, &mut pos);
        let hd_reserved = utils::read(stream, little_endian, &mut pos);
        let hd_start_angle_rad = utils::read(stream, little_endian, &mut pos);
        let hd_start_distance_m = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            HDBLOCK {
                hd_dg_first,
                hd_fh_first,
                hd_ch_first,
                hd_at_first,
                hd_ev_first,
                hd_md_comment,
                hd_start_time_ns,
                hd_tz_offset_min,
                hd_dst_offset_min,
                hd_time_flags,
                hd_time_class,
                hd_flags,
                hd_reserved,
                hd_start_angle_rad,
                hd_start_distance_m,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct MDBLOCK {
	md_data: String,
}
impl Block for MDBLOCK{
	fn new() -> Self {
        Self {
			md_data: "".to_string(),
		}
    }
    fn default() -> Self {
        Self {
			md_data: "".to_string(),
		}
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {

		let (pos, header) = BlockHeader::read(stream, position, little_endian);
		
		if !utils::eq(
			&header.id, 
			&"##MD".as_bytes()
		) {
			panic!("Error type incorrect");
		}


		let mut md_data_temp = "";
		unsafe {
			md_data_temp = str_from_u8_nul_utf8_unchecked(&stream[pos..(pos+header.length as usize - 10)]);
		}

		let md_data = md_data_temp.to_string();

        (pos+md_data.len(), Self {
			md_data,
		})
    }
}

pub unsafe fn str_from_u8_nul_utf8_unchecked(utf8_src: &[u8]) -> &str {
    let nul_range_end = utf8_src.iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len()); // default to length if no `\0` present
    ::std::str::from_utf8_unchecked(&utf8_src[0..nul_range_end])
}

#[derive(Debug, Clone)]
struct TXBLOCK {
	tx_data: String,
}
impl Block for TXBLOCK {
    fn new() -> Self {
        Self {
			tx_data: String::new(),
		}
    }
    fn default() -> Self {
        Self {
			tx_data: String::new(),
		}
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
		let (pos, header) = BlockHeader::read(stream, position, little_endian);
		
		if !utils::eq(
			&header.id, 
			&"##MD".as_bytes()
		) {
			panic!("Error type incorrect");
		}


		let mut tx_data_temp = "";
		unsafe {
			tx_data_temp = str_from_u8_nul_utf8_unchecked(&stream[pos..(pos+header.length as usize - 10)]);
		}

		let tx_data = tx_data_temp.to_string();

        (pos+header.length as usize, Self {
			tx_data,
		})

    }
}

#[derive(Debug, Clone)]
struct FHBLOCK {
	fh_fh_next: u64, 
	fh_md_comment: u64, 
	fh_time_ns: u64, 
	fh_tz_offset_min: i16, 
	fh_dst_offset_min: i16, 
	fh_time_flags: u8
}
impl Block for FHBLOCK {
    fn new() -> Self {
        Self {
			fh_fh_next: 0_u64, 
			fh_md_comment: 0_u64, 
			fh_time_ns: 0_u64, 
			fh_tz_offset_min: 0_i16, 
			fh_dst_offset_min: 0_i16, 
			fh_time_flags: 0_u8
		}
    }
    fn default() -> Self {
        Self {
		fh_fh_next: 0_u64, 
		fh_md_comment: 0_u64, 
		fh_time_ns: 0_u64, 
		fh_tz_offset_min: 0_i16, 
		fh_dst_offset_min: 0_i16, 
		fh_time_flags: 0_u8
		}
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {

		let (pos, header) = BlockHeader::read(stream, position, little_endian);
		let (mut pos, address) = link_extract(stream, pos, little_endian, header.link_count);

		let fh_fh_next = address.remove(0);
		let fh_md_comment = address.remove(0);

		let fh_time_ns  = utils::read(stream, little_endian, &mut pos);
		let fh_tz_offset_min  = utils::read(stream, little_endian, &mut pos);
		let fh_dst_offset_min  = utils::read(stream, little_endian, &mut pos);
		let fh_time_flags = utils::read(stream, little_endian, &mut pos);

        (pos, Self {
			fh_fh_next, 
			fh_md_comment, 
			fh_time_ns, 
			fh_tz_offset_min, 
			fh_dst_offset_min, 
			fh_time_flags
		})
    }
}

enum ChannelHierarchyType {
	Group, 
	Function, 
	Structure, 
	MapList, 
	FunctionInput, 
	FunctionOutput, 
	FunctionLocal, 
	FunctionCalDef, 
	FunctionCalRef, 
}

impl ChannelHierarchyType {
	fn new(ch_type: u8) -> Self{
		match ch_type {
			0 => Self::Group, 
			1 => Self::Function, 
			2 => Self::Structure, 
			3 => Self::MapList, 
			4 => Self::FunctionInput, 
			5 => Self::FunctionOutput, 
			6 => Self::FunctionLocal, 
			7 => Self::FunctionCalDef, 
			8 => Self::FunctionCalRef, 
			_ => panic!("Unknown channel type")
		}
	}
}

struct CHBLOCK {
	ch_ch_next: u64, 
	ch_ch_first: u64, 
	ch_tx_name: u64, 
	ch_md_comment: u64, 
	ch_element: Vec<u64>,
	ch_element_count: u32, 
	ch_type: ChannelHierarchyType, 
}
impl Block for CHBLOCK {
	fn new() -> Self{
		Self{ch_ch_next: 0_u64, 
		ch_ch_first: 0_u64, 
		ch_tx_name: 0_u64, 
		ch_md_comment: 0_u64, 
		ch_element: Vec::new(),
		ch_element_count: 0_u32, 
		ch_type: ChannelHierarchyType::Function, }
	}
	fn default() -> Self{		Self{ch_ch_next: 0_u64, 
		ch_ch_first: 0_u64, 
		ch_tx_name: 0_u64, 
		ch_md_comment: 0_u64, 
		ch_element: Vec::new(),
		ch_element_count: 0_u32, 
		ch_type: ChannelHierarchyType::Function, }}
	fn read(stream: &[u8], position: usize,little_endian: bool) -> (usize, Self){
		let (pos, header) = BlockHeader::read(stream, position, little_endian);
		let (mut pos, address)  = link_extract(stream, pos, little_endian, header.link_count);



		let ch_element_count = utils::read(stream, little_endian, &mut pos);
		let ch_type = ChannelHierarchyType::new( utils::read(stream, little_endian, &mut pos));

		let ch_ch_next = address.remove(0);
		let ch_ch_first = address.remove(0);
		let ch_tx_name = address.remove(0);
		let ch_md_comment = address.remove(0);
		let mut ch_element = Vec::with_capacity(ch_element_count as usize * 3 );
		for i in 0..(ch_element_count * 3) {
			ch_element.push(address.remove(0));
		}

		(pos, Self {
			ch_ch_next,
			ch_ch_first,
			ch_tx_name,
			ch_md_comment,
			ch_element,
			ch_element_count,
			ch_type,

		})
	}
}
#[derive(Debug, Clone)]
struct ATBLOCK {
    //id: [u8; 4],
    //reserved0: [u8; 4],
    //block_len: u64,
    //links_nr: u64,
    next_at_addr: u64,
    file_name_addr: u64,
    mime_addr: u64,
    comment_addr: u64,
    flags: u16,
    creator_index: u16,
    //reserved1: [u8; 4],
    md5_sum: [u8; 16],
    original_size: u64,
    embedded_size: u64,
    embedded_data: Vec<u8>,
}

impl Block for ATBLOCK {
    fn new() -> Self {
        Self {
            next_at_addr: 0,
            file_name_addr: 0,
            mime_addr: 0,
            comment_addr: 0,
            flags: 0,
            creator_index: 0,
            //reserved1: [0; 4],
            md5_sum: [0; 16],
            original_size: 0,
            embedded_size: 0,
            embedded_data: Vec::new(),
        }
    }
    fn default() -> Self {
        Self {
            next_at_addr: 0,
            file_name_addr: 0,
            mime_addr: 0,
            comment_addr: 0,
            flags: 0,
            creator_index: 0,
            //reserved1: [0; 4],
            md5_sum: [0; 16],
            original_size: 0,
            embedded_size: 0,
            embedded_data: Vec::new(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id[..], &[b'#', b'#', b'A', b'T']) {
            panic!("Error: block id doesn't match Attachment Block");
        }

        let (mut pos, addresses) = link_extract(stream, pos, little_endian, header.link_count);

        let next_at_addr = addresses[0];
        let file_name_addr = addresses[1];
        let mime_addr = addresses[2];
        let comment_addr = addresses[3];

        let flags = utils::read(stream, little_endian, &mut pos);
        let creator_index = utils::read(stream, little_endian, &mut pos);
        let reserved1: [u8; 4] = utils::read(stream, little_endian, &mut pos);
        let md5_sum = utils::read(stream, little_endian, &mut pos);
        let original_size = utils::read(stream, little_endian, &mut pos);
        let embedded_size = utils::read(stream, little_endian, &mut pos);
        let embedded_data = stream[pos..pos + embedded_size as usize].to_vec();

        (
            pos,
            Self {
                //id: header.id,
                //reserved0: header.reserved0,
                //block_len: header.length,
                //links_nr: header.link_count,
                next_at_addr,
                file_name_addr,
                mime_addr,
                comment_addr,
                flags,
                creator_index,
                //reserved1,
                md5_sum,
                original_size,
                embedded_size,
                embedded_data,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct EVBlock {
	ev_ev_next: u64, 
	ev_ev_parent: u64, 
	ev_ev_range: u64, 
	ev_tx_name: u64,
	ev_md_comment: u64, 
	ev_scope: Vec<u64>,
	ev_at_reference: Vec<u64>,
	ev_type: EventType, 
	ev_sync_type: EventSyncType, 
	ev_range_type: RangeType,
	ev_cause: EventCause, 
	ev_flags: u8, 
	ev_scope_count: u32, 
	ev_attachment_count: u16, 
	ev_creator_index: u16, 
	ev_sync_base_value: i64, 
	ev_sync_factor: f64, 
}

impl Block for EVBlock {
    fn new() -> Self {
        Self {
			ev_ev_next: 0_u64, 
			ev_ev_parent: 0_u64, 
			ev_ev_range: 0_u64, 
			ev_tx_name: 0_u64,
			ev_md_comment: 0_u64, 
			ev_scope: Vec::new(),
			ev_at_reference: Vec::new(),
			ev_type: EventType::AcquistionInterrupt, 
			ev_sync_type: EventSyncType::Index, 
			ev_range_type: RangeType::Point,
			ev_cause: EventCause::Error, 
			ev_flags: 0_u8, 
			ev_scope_count: 0_u32, 
			ev_attachment_count: 0_u16, 
			ev_creator_index:0_u16, 
			ev_sync_base_value: 0_i64, 
			ev_sync_factor: 0_f64, 
		}
    }
    fn default() -> Self {
        Self {
			ev_ev_next: 0_u64, 
			ev_ev_parent: 0_u64, 
			ev_ev_range: 0_u64, 
			ev_tx_name: 0_u64,
			ev_md_comment: 0_u64, 
			ev_scope: Vec::new(),
			ev_at_reference: Vec::new(),
			ev_type: EventType::AcquistionInterrupt, 
			ev_sync_type: EventSyncType::Index, 
			ev_range_type: RangeType::Point,
			ev_cause: EventCause::Error, 
			ev_flags: 0_u8, 
			ev_scope_count: 0_u32, 
			ev_attachment_count: 0_u16, 
			ev_creator_index:0_u16, 
			ev_sync_base_value: 0_i64, 
			ev_sync_factor: 0_f64, 

		}
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {

		let (pos, header) = BlockHeader::read(stream, position, little_endian);
		let (mut pos, address) = link_extract(stream, pos, little_endian, header.link_count);

		let ev_type = EventType::new(utils::read(stream, little_endian, &mut pos));
		let ev_sync_type = EventSyncType::new(utils::read(stream, little_endian, &mut pos));
		let ev_range_type = RangeType::new(utils::read(stream, little_endian, &mut pos));
		let ev_cause = EventCause::new(utils::read(stream, little_endian, &mut pos));
		let ev_flags = utils::read(stream, little_endian, &mut pos);
		
		let ev_reserved : [u8; 3] = utils::read(stream, little_endian, &mut pos);

		let ev_scope_count = utils::read(stream, little_endian, &mut pos);
		let ev_attachment_count = utils::read(stream, little_endian, &mut pos);
		let ev_creator_index = utils::read(stream, little_endian, &mut pos);
		let ev_sync_base_value = utils::read(stream, little_endian, &mut pos);
		let ev_sync_factor = utils::read(stream, little_endian, &mut pos);


		let ev_ev_next = address.remove(0);
		let ev_ev_parent = address.remove(0);
		let ev_ev_range = address.remove(0);
		let ev_tx_name = address.remove(0);
		let ev_md_comment = address.remove(0);
		let mut ev_scope = Vec::new();
		for i in 0..ev_scope_count  {
			ev_scope.push(address.remove(0));
		}
		let mut ev_at_reference = Vec::new();
		for i in 0..ev_attachment_count {
			ev_at_reference.push(address.remove(0));
		}

        (pos, Self {
			ev_ev_next,
			ev_ev_parent,
			ev_ev_range,
			ev_tx_name,
			ev_md_comment,
			ev_scope,
			ev_at_reference,
			ev_type,
			ev_sync_type,
			ev_range_type,
			ev_cause,
			ev_flags,
			ev_scope_count,
			ev_attachment_count,
			ev_creator_index,
			ev_sync_base_value,
			ev_sync_factor,
		})
    }
}

#[derive(Debug, Clone)]

enum EventType{
	Recording, 
	RecordingInterrupt, 
	AcquistionInterrupt, 
	StartRecordingTrigger, 
	StopRecordingTrigger, 
	Trigger, 
	Marker,
}

impl EventType {
	fn new(ev_type: u8) -> Self {
		match ev_type {
			0 => Self::Recording, 
			1 => Self::RecordingInterrupt, 
			2 => Self::AcquistionInterrupt, 
			3 => Self::StartRecordingTrigger, 
			4 => Self::StopRecordingTrigger, 
			5 => Self::Trigger, 
			6 => Self::Marker,
			_ => panic!("Error with Event Type")
		}
	}
}

#[derive(Debug, Clone)]

enum EventSyncType{
	Seconds, 
	Radians, 
	Meters, 
	Index,
}

impl EventSyncType {
	fn new(ev_sync: u8) -> Self {
		match ev_sync {
			1 => Self::Seconds, 
			2 => Self::Radians, 
			3 => Self::Meters, 
			4 => Self::Index,
			_ => panic!("Error Event Sync Type")
		}
	}
}

#[derive(Debug, Clone)]

enum RangeType{
	Point, 
	RangeBegin, 
	RangeEnd, 
}

impl RangeType {
	fn new(ev_range: u8) -> Self {
		match ev_range {
			0 => Self::Point, 
			1 => Self::RangeBegin, 
			2 => Self::RangeEnd, 
			_ => panic!("Error Range Type")
		}
	}
}

#[derive(Debug, Clone)]
enum EventCause{
	Other, 
	Error, 
	Tool, 
	Script, 
	User,
}

impl EventCause {
	fn new(ev_cause: u8) -> Self{
		match ev_cause{
			0 => Self::Other, 
			1 => Self::Error, 
			2 => Self::Tool, 
			3 => Self::Script, 
			4 => Self::User,
			_ => panic!("Error Event cause")
		}
	}
}

#[derive(Debug, Clone)]
struct DGBLOCK {
	dg_dg_next: u64, 
	dg_cg_first: u64, 
	dg_data: u64, 
	dg_md_comment: u64, 
	dg_rec_id_size: u8
}
impl Block for DGBLOCK {
    fn new() -> Self {
        Self {	
			dg_dg_next: 0_u64, 
			dg_cg_first: 0_u64, 
			dg_data: 0_u64, 
			dg_md_comment: 0_u64, 
			dg_rec_id_size: 0_u8}
    }
    fn default() -> Self {
		Self {	
			dg_dg_next: 0_u64, 
			dg_cg_first: 0_u64, 
			dg_data: 0_u64, 
			dg_md_comment: 0_u64, 
			dg_rec_id_size: 0_u8}
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {

		let (pos, header) = BlockHeader::read(stream, position, little_endian);
		let (mut pos, address) = link_extract(stream, pos, little_endian, header.link_count);

		let dg_rec_id_size = utils::read(stream, little_endian, &mut pos);
		let dg_reserved: [u8; 7] = utils::read(stream, little_endian, &mut pos);

		let dg_dg_next = address.remove(0);
		let dg_cg_first = address.remove(0);
		let dg_data = address.remove(0);
		let dg_md_comment = address.remove(0);

        (pos, Self {
			dg_dg_next, 
			dg_cg_first, 
			dg_data, 
			dg_md_comment, 
			dg_rec_id_size,
		})
    }
}

#[derive(Debug, Clone)]
struct CGBLOCK {
    //id: [u8; 4],        //- bytes : block ID; always b'##CG'
    //reserved0: u64,     //- int : reserved bytes
    //block_len: u64,     //- int : block bytes size
    //links_nr: u64,      //- int : number of links
    next_cg_addr: u64,  //- int : next channel group address
    first_ch_addr: u64, //- int : address of first channel of this channel group
    acq_name_addr: u64, //- int : address of TextBLock that contains the channel
    //group acquisition name
    acq_source_addr: u64, //- int : address of SourceInformation that contains the
    //channel group source
    first_sample_reduction_addr: u64, // - int : address of first SRBLOCK; this is
    //considered 0 since sample reduction is not yet supported
    comment_addr: u64, //- int : address of TXBLOCK/MDBLOCK that contains the
    //channel group comment
    record_id: u64,     //- int : record ID for the channel group
    cycles_nr: u64,     //- int : number of cycles for this channel group
    flags: u64,         //- int : channel group flags
    path_separator: u8, //- int : ordinal for character used as path separator
    //reserved1: u64,       //- int : reserved bytes
    samples_byte_nr: u64, //- int : number of bytes used for channels samples in
    //the record for this channel group; this does not contain the invalidation
    //bytes
    invalidation_bytes_nr: u64, // - int : number of bytes used for invalidation
                                // bits by this channel group

                                //Other attributes
                                //acq_name: u64,   // - str : acquisition name
                                //acq_source: u64, //- SourceInformation : acquisition source information
                                //address: u64,    //- int : channel group address
                                //comment: u64,    //- str : channel group comment
}
impl Block for CGBLOCK {
    fn new() -> Self {
        CGBLOCK {
            next_cg_addr: 0,
            first_ch_addr: 0,
            acq_name_addr: 0,
            acq_source_addr: 0,
            first_sample_reduction_addr: 0,
            comment_addr: 0,
            record_id: 0,
            cycles_nr: 0,
            flags: 0,
            path_separator: 0,
            samples_byte_nr: 0,
            invalidation_bytes_nr: 0,
        }
    }
    fn default() -> Self {
        CGBLOCK {
            next_cg_addr: 0,
            first_ch_addr: 0,
            acq_name_addr: 0,
            acq_source_addr: 0,
            first_sample_reduction_addr: 0,
            comment_addr: 0,
            record_id: 0,
            cycles_nr: 0,
            flags: 0,
            path_separator: 0,
            samples_byte_nr: 0,
            invalidation_bytes_nr: 0,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (mut pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id[..], &[b'#', b'#', b'C', b'G']) {
            panic!("Error: Channel group wrong id");
        }

        let (mut pos, addresses) = link_extract(stream, pos, little_endian, header.link_count);

        let next_cg_addr = addresses.remove(0);
        let first_ch_addr = addresses.remove(0);
        let acq_name_addr = addresses.remove(0);
        let acq_source_addr = addresses.remove(0);
        let first_sample_reduction_addr = addresses.remove(0);
        let comment_addr = addresses.remove(0);

        let record_id = utils::read(stream, little_endian, &mut pos);
        let cycles_nr = utils::read(stream, little_endian, &mut pos);
        let flags = utils::read(stream, little_endian, &mut pos);
        let path_separator = utils::read(stream, little_endian, &mut pos);
        let reserved1: [u8; 4] = utils::read(stream, little_endian, &mut pos);
        let samples_byte_nr = utils::read(stream, little_endian, &mut pos);
        let invalidation_bytes_nr = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            CGBLOCK {
                // id,
                // reserved0,
                // block_len,
                // links_nr,
                next_cg_addr,
                first_ch_addr,
                acq_name_addr,
                acq_source_addr,
                first_sample_reduction_addr,
                comment_addr,
                record_id,
                cycles_nr,
                flags,
                path_separator,
                //reserved1,
                samples_byte_nr,
                invalidation_bytes_nr,
                // acq_name,
                // comment,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct SIBLOCK {
	si_tx_name: u64, 
	si_tx_path: u64, 
	si_md_comment: u64, 
	si_type: SourceType, 
	si_bus_type: BusType, 
	si_flags: u8, 

}
impl Block for SIBLOCK {
    fn new() -> Self {
        SIBLOCK {
			si_tx_name: 0_u64, 
			si_tx_path: 0_u64, 
			si_md_comment: 0_u64, 
			si_type: SourceType::Bus, 
			si_bus_type: BusType::CAN, 
			si_flags: 0_u8, 
		}
    }
    fn default() -> Self {
		SIBLOCK {
			si_tx_name: 0_u64, 
			si_tx_path: 0_u64, 
			si_md_comment: 0_u64, 
			si_type: SourceType::Bus, 
			si_bus_type: BusType::CAN, 
			si_flags: 0_u8, 
		}
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {

		let (pos, header) = BlockHeader::read(stream, position, little_endian);
		let (mut pos, address) = link_extract(stream, pos ,little_endian, header.link_count);

		let si_tx_name = address.remove(0); 
		let si_tx_path = address.remove(0); 
		let si_md_comment = address.remove(0);
		
		let si_type = SourceType::new(utils::read(stream, little_endian, &mut pos));
		let si_bus_type = BusType::new(utils::read(stream, little_endian, &mut pos));
		let si_flags = utils::read(stream, little_endian, &mut pos);

		let si_reserved: [u8; 5] = utils::read(stream , little_endian, &mut pos);

        (pos, SIBLOCK {
			si_tx_name,
			si_tx_path,
			si_md_comment,
			si_type,
			si_bus_type,
			si_flags,
		})
    }
}

#[derive(Debug, Clone)]
enum SourceType{
	Other, 
	ECU, 
	Bus, 
	IO, 
	Tool, 
	User,
}

impl SourceType {
	fn new(source: u8) -> Self{
		match source {
			0 => Self::Other, 
			1 => Self::ECU, 
			2 => Self::Bus, 
			3 => Self::IO, 
			4 => Self::Tool, 
			5 => Self::User,
			_ => panic!("Error source type")
		}
	}
}

#[derive(Debug, Clone)]
enum BusType{
	None, 
	Other, 
	CAN, 
	LIN, 
	MOST, 
	FlexRay, 
	KLine, 
	Ethernet, 
	USB,
}

impl BusType {
	fn new(source: u8) -> Self{
		match source {
			0 => Self::None, 
			1 => Self::Other, 
			2 => Self::CAN, 
			3 => Self::LIN, 
			4 => Self::MOST, 
			5 => Self::FlexRay, 
			6 => Self::KLine, 
			7 => Self::Ethernet, 
			8 => Self::USB,
			_ => panic!("Error bus type")
		}
	}
}

#[derive(Debug, Clone)]
struct CNBLOCK {
    //id: [u8; 4],        //block ID; always b'##CN'
    //reserved0: u32,      //reserved bytes
    //block_len: u64,      //block bytes size
    //links_nr: u64,       //number of links
    next_ch_addr: u64,   //next ATBLOCK address
    component_addr: u64, //address of first channel in case of structure channel
    //   composition, or ChannelArrayBlock in case of arrays
    //   file name
    name_addr: u64,       //address of TXBLOCK that contains the channel name
    source_addr: u64,     //address of channel source block
    conversion_addr: u64, //address of channel conversion block
    data_block_addr: u64, //address of signal data block for VLSD channels
    unit_addr: u64,       //address of TXBLOCK that contains the channel unit
    comment_addr: u64,    //address of TXBLOCK/MDBLOCK that contains the
    //   channel comment
    attachment_addr: Vec<u64>, //address of N:th ATBLOCK referenced by the
    //   current channel; if no ATBLOCK is referenced there will be no such key:value
    //   pair
    default_X_dg_addr: u64, //address of DGBLOCK where the default X axis
    //   channel for the current channel is found; this key:value pair will not
    //   exist for channels that don't have a default X axis
    default_X_cg_addr: u64, //address of CGBLOCK where the default X axis
    //   channel for the current channel is found; this key:value pair will not
    //   exist for channels that don't have a default X axis
    default_X_ch_addr: u64, //address of default X axis
    //   channel for the current channel; this key:value pair will not
    //   exist for channels that don't have a default X axis
    channel_type: ChannelType, //integer code for the channel type
    sync_type: SyncType,       //integer code for the channel's sync type
    data_type: DataType,       //integer code for the channel's data type
    bit_offset: u8,            //bit offset
    byte_offset: u32,          //byte offset within the data record
    bit_count: u32,            //channel bit count
    flags: u32,                //CNBLOCK flags
    pos_invalidation_bit: u32, //invalidation bit position for the current
    //   channel if there are invalidation bytes in the data record
    precision: u8, //integer code for the precision
    //reserved1: u8,        //reserved bytes
    min_raw_value: f64,   //min raw value of all samples
    max_raw_value: f64,   //max raw value of all samples
    lower_limit: f64,     //min physical value of all samples
    upper_limit: f64,     //max physical value of all samples
    lower_ext_limit: f64, //min physical value of all samples
    upper_ext_limit: f64, //max physical value of all samples

                          // Other attributes
                          // address: u8,             //channel address
                          // attachments: Vec<usize>, //list of referenced attachment blocks indexes;
                          // //   the index reference to the attachment block index
                          // comment: String,     // channel comment
                          // conversion: CCBLOCK, // channel conversion; None if the
                          // //   channel has no conversion
                          // display_name: String, // channel display name; this is extracted from the
                          // //   XML channel comment
                          // name: String,              //channel name
                          // source: SourceInformation, // channel source information; None if
                          // //   the channel has no source information
                          // unit: String, // channel unit
}

impl Block for CNBLOCK {
    fn new() -> Self {
        CNBLOCK {
            next_ch_addr: 0,
            component_addr: 0,
            name_addr: 0,
            source_addr: 0,
            conversion_addr: 0,
            data_block_addr: 0,
            unit_addr: 0,
            comment_addr: 0,
            attachment_addr: Vec::new(),
            default_X_dg_addr: 0,
            default_X_cg_addr: 0,
            default_X_ch_addr: 0,
            channel_type: ChannelType::FixedLengthChannel,
            sync_type: SyncType::Angle,
            data_type: DataType::ByteArray,
            bit_offset: 0,
            byte_offset: 0,
            bit_count: 0,
            flags: 0,
            pos_invalidation_bit: 0,
            precision: 0,
            min_raw_value: 0.0,
            max_raw_value: 0.0,
            lower_limit: 0.0,
            upper_limit: 0.0,
            lower_ext_limit: 0.0,
            upper_ext_limit: 0.0,
        }
    }
    fn default() -> Self {
        CNBLOCK {
            next_ch_addr: 0,
            component_addr: 0,
            name_addr: 0,
            source_addr: 0,
            conversion_addr: 0,
            data_block_addr: 0,
            unit_addr: 0,
            comment_addr: 0,
            attachment_addr: Vec::new(),
            default_X_dg_addr: 0,
            default_X_cg_addr: 0,
            default_X_ch_addr: 0,
            channel_type: ChannelType::FixedLengthChannel,
            sync_type: SyncType::Angle,
            data_type: DataType::ByteArray,
            bit_offset: 0,
            byte_offset: 0,
            bit_count: 0,
            flags: 0,
            pos_invalidation_bit: 0,
            precision: 0,
            min_raw_value: 0.0,
            max_raw_value: 0.0,
            lower_limit: 0.0,
            upper_limit: 0.0,
            lower_ext_limit: 0.0,
            upper_ext_limit: 0.0,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, &[b'#', b'#', b'C', b'N']) {
            panic!("Error: Incorrect channel id");
        }

        let (mut pos, addresses) = link_extract(stream, pos, little_endian, header.link_count);

        let channel_type = ChannelType::new(utils::read(stream, little_endian, &mut pos));
        let sync_type = SyncType::new(utils::read(stream, little_endian, &mut pos));
        let data_type = DataType::new(utils::read(stream, little_endian, &mut pos));

        let bit_offset = utils::read(stream, little_endian, &mut pos);
        let byte_offset = utils::read(stream, little_endian, &mut pos);
        let bit_count = utils::read(stream, little_endian, &mut pos);
        let flags = utils::read(stream, little_endian, &mut pos);
        let invalidation_bit_pos = utils::read(stream, little_endian, &mut pos);
        let precision = utils::read(stream, little_endian, &mut pos);
        let reserved1: u8 = utils::read(stream, little_endian, &mut pos);
        let attachment_nr: u16 = utils::read(stream, little_endian, &mut pos);
        let min_raw_value = utils::read(stream, little_endian, &mut pos);
        let max_raw_value = utils::read(stream, little_endian, &mut pos);
        let lower_limit = utils::read(stream, little_endian, &mut pos);
        let upper_limit = utils::read(stream, little_endian, &mut pos);
        let lower_ext_limit = utils::read(stream, little_endian, &mut pos);
        let upper_ext_limit = utils::read(stream, little_endian, &mut pos);

        let next_ch_addr = addresses.remove(0);
        let component_addr = addresses.remove(0);
        let name_addr = addresses.remove(0);
        let source_addr = addresses.remove(0);
        let conversion_addr = addresses.remove(0);
        let data_block_addr = addresses.remove(0);
        let unit_addr = addresses.remove(0);
        let comment_addr = addresses.remove(0);

        let mut attachment_addr = Vec::with_capacity(attachment_nr as usize);
        for i in 0..attachment_nr {
            attachment_addr.push(addresses.remove(0));
        }

        let mut default_x = Vec::with_capacity(3);
        for i in 0..3 {
            default_x.push(addresses.remove(0));
        }

        (
            1,
            CNBLOCK {
                next_ch_addr,
                component_addr,
                name_addr,
                source_addr,
                conversion_addr,
                data_block_addr,
                unit_addr,
                comment_addr,
                attachment_addr,
                default_X_dg_addr: default_x[0],
                default_X_cg_addr: default_x[1],
                default_X_ch_addr: default_x[2],
                channel_type,
                sync_type,
                data_type,
                bit_offset,
                byte_offset,
                bit_count,
                flags,
                pos_invalidation_bit: invalidation_bit_pos,
                precision,
                //reserved1,
                min_raw_value,
                max_raw_value,
                lower_limit,
                upper_limit,
                lower_ext_limit,
                upper_ext_limit,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct CCBLOCK {
    name_addr: u64,
    unit_addr: u64,
    comment_addr: u64,
    inv_conv_addr: u64,
    cc_ref: Vec<u64>,

    conversion_type: CCType,
    precision: u8,
    flags: u16,
    ref_param_nr: u16,
    val_param_nr: u16,
    min_phy_value: f64,
    max_phy_value: f64,
    cc_val: Vec<f64>,
}
impl Block for CCBLOCK {
    fn new() -> Self {
        Self {
            name_addr: 0,
            unit_addr: 0,
            comment_addr: 0,
            inv_conv_addr: 0,
            cc_ref: Vec::new(),

            conversion_type: CCType::Direct,
            precision: 0,
            flags: 0,
            ref_param_nr: 0,
            val_param_nr: 0,
            min_phy_value: 0.0,
            max_phy_value: 0.0,
            cc_val: Vec::new(),
        }
    }
    fn default() -> Self {
        Self {
            name_addr: 0,
            unit_addr: 0,
            comment_addr: 0,
            inv_conv_addr: 0,
            cc_ref: Vec::new(),

            conversion_type: CCType::Direct,
            precision: 0,
            flags: 0,
            ref_param_nr: 0,
            val_param_nr: 0,
            min_phy_value: 0.0,
            max_phy_value: 0.0,
            cc_val: Vec::new(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;

        let (mut pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id[..], &[b'#', b'#', b'C', b'C']) {
            panic!("Error: id incorrect");
        }

        let name_addr = utils::read(stream, little_endian, &mut pos);
        let unit_addr = utils::read(stream, little_endian, &mut pos);
        let comment_addr = utils::read(stream, little_endian, &mut pos);
        let inv_conv_addr = utils::read(stream, little_endian, &mut pos);

        let cc_ref_length = (header.link_count - 4) as usize;
        let mut cc_ref = Vec::new();

        for i in 0..cc_ref_length {
            cc_ref.push(utils::read(stream, little_endian, &mut pos));
        }

        let conversion_type = CCType::new(utils::read(stream, little_endian, &mut pos));
        let precision = utils::read(stream, little_endian, &mut pos);
        let flags = utils::read(stream, little_endian, &mut pos);
        let ref_param_nr = utils::read(stream, little_endian, &mut pos);
        let val_param_nr = utils::read(stream, little_endian, &mut pos);
        let min_phy_value = utils::read(stream, little_endian, &mut pos);
        let max_phy_value = utils::read(stream, little_endian, &mut pos);

        let mut cc_val = Vec::new();
        for i in 0..val_param_nr {
            cc_val.push(utils::read(stream, little_endian, &mut pos));
        }

        // Check ref count
        assert_eq!(ref_param_nr as usize, cc_ref.len());

        (
            pos,
            Self {
                name_addr,
                unit_addr,
                comment_addr,
                inv_conv_addr,
                conversion_type,
                cc_ref,
                precision,
                flags,
                ref_param_nr,
                val_param_nr,
                min_phy_value,
                max_phy_value,
                cc_val,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct CABLOCK {
    ca_composition: u64,
    ca_data: Vec<u64>,
    ca_dynamic_size: Vec<u64>,
    ca_input_quantity: Vec<u64>,
    ca_output_quantity: Vec<u64>,
    ca_comparison_quantity: Vec<u64>,
    ca_cc_axis_conversion: Vec<u64>,
    ca_axis: Vec<u64>,
    ca_type: u8,
    ca_storage: u8,
    ca_ndim: u16,
    ca_flags: u32,
    ca_byte_offset_base: i32,
    ca_inval_bit_pos_base: u32,
    ca_dim_size: Vec<u64>,
    ca_axis_value: Vec<f64>,
    ca_cycle_count: Vec<u64>,
}
impl Block for CABLOCK {
    fn new() -> Self {
        CABLOCK {
            ca_composition: 0,
            ca_data: Vec::new(),
            ca_dynamic_size: Vec::new(),
            ca_input_quantity: Vec::new(),
            ca_output_quantity: Vec::new(),
            ca_comparison_quantity: Vec::new(),
            ca_cc_axis_conversion: Vec::new(),
            ca_axis: Vec::new(),
            ca_type: 0,
            ca_storage: 0,
            ca_ndim: 0,
            ca_flags: 0,
            ca_byte_offset_base: 0,
            ca_inval_bit_pos_base: 0,
            ca_dim_size: Vec::new(),
            ca_axis_value: Vec::new(),
            ca_cycle_count: Vec::new(),
        }
    }
    fn default() -> Self {
        CABLOCK {
            ca_composition: 0,
            ca_data: Vec::new(),
            ca_dynamic_size: Vec::new(),
            ca_input_quantity: Vec::new(),
            ca_output_quantity: Vec::new(),
            ca_comparison_quantity: Vec::new(),
            ca_cc_axis_conversion: Vec::new(),
            ca_axis: Vec::new(),
            ca_type: 0,
            ca_storage: 0,
            ca_ndim: 0,
            ca_flags: 0,
            ca_byte_offset_base: 0,
            ca_inval_bit_pos_base: 0,
            ca_dim_size: Vec::new(),
            ca_axis_value: Vec::new(),
            ca_cycle_count: Vec::new(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (mut pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id[..], &[b'#', b'#', b'C', b'A']) {
            panic!("Error: id CABLOCK");
        }

        let (mut pos, addresses) = link_extract(stream, pos, little_endian, header.link_count);

        let ca_type = utils::read(stream, little_endian, &mut pos);
        let ca_storage = utils::read(stream, little_endian, &mut pos);
        let ca_ndim = utils::read(stream, little_endian, &mut pos);
        let ca_flags = utils::read(stream, little_endian, &mut pos);
        let ca_byte_offset_base = utils::read(stream, little_endian, &mut pos);
        let ca_inval_bit_pos_base = utils::read(stream, little_endian, &mut pos);

        let D = ca_ndim as usize;

        let ca_dim_size = Vec::new();
        for i in 0..D {
            ca_dim_size.push(utils::read(stream, little_endian, &mut pos));
        }

        let nd_sum = ca_dim_size.iter().sum();
        let nd_prod = ca_dim_size.iter().product();

        let ca_axis_value = Vec::new();
        for i in 0..nd_sum {
            ca_axis_value.push(utils::read(stream, little_endian, &mut pos));
        }

        let ca_cycle_count = Vec::new();
        for i in 0..nd_prod {
            ca_cycle_count.push(utils::read(stream, little_endian, &mut pos));
        }

        let ca_composition = addresses.remove(0);
        let mut ca_data = Vec::new();
        for i in 0..nd_prod {
            ca_data.push(addresses.remove(0));
        }
        let ca_dynamic_size = Vec::new();
        for i in 0..(3 * D) {
            ca_dynamic_size.push(addresses.remove(0));
        }
        let mut ca_input_quantity = Vec::new();
        for i in 0..(3 * D) {
            ca_input_quantity.push(addresses.remove(0));
        }
        let mut ca_output_quantity = Vec::new();
        for i in 0..3 {
            ca_output_quantity.push(addresses.remove(0));
        }
        let ca_comparison_quantity = Vec::new();
        for i in 0..3 {
            ca_comparison_quantity.push(addresses.remove(0));
        }
        let mut ca_cc_axis_conversion = Vec::new();
        for i in 0..D {
            ca_cc_axis_conversion.push(addresses.remove(0));
        }
        let mut ca_axis = Vec::new();
        for i in 0..(3 * D) {
            ca_axis.push(addresses.remove(0));
        }

        (
            pos,
            CABLOCK {
                ca_composition,
                ca_data,
                ca_dynamic_size,
                ca_input_quantity,
                ca_output_quantity,
                ca_comparison_quantity,
                ca_cc_axis_conversion,
                ca_axis,
                ca_type,
                ca_storage,
                ca_ndim,
                ca_flags,
                ca_byte_offset_base,
                ca_inval_bit_pos_base,
                ca_dim_size,
                ca_axis_value,
                ca_cycle_count,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct DTBlock {}
impl Block for DTBlock {
    fn new() -> Self {
        Self {}
    }
    fn default() -> Self {
        Self {}
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        (1, Self {})
    }
}

#[derive(Debug, Clone)]
struct SRBLOCK {}

#[derive(Debug, Clone)]
struct RDBLOCK {}

#[derive(Debug, Clone)]
struct SDBLOCK {}

#[derive(Debug, Clone)]
struct DLBLOCK {}
impl Block for DLBLOCK {
    fn new() -> Self {
        Self {}
    }
    fn default() -> Self {
        Self {}
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        (1, Self {})
    }
}

#[derive(Debug, Clone)]
struct DZBlock {}
impl Block for DZBlock {
    fn new() -> Self {
        Self {}
    }
    fn default() -> Self {
        Self {}
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        (1, Self {})
    }
}

#[derive(Debug, Clone)]
struct HLBLOCK {}
impl Block for HLBLOCK {
    fn new() -> Self {
        Self {}
    }
    fn default() -> Self {
        Self {}
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        (1, Self {})
    }
}

#[derive(Debug, Clone)]
enum ChannelType {
    FixedLengthChannel,
    VariableLengthChannel,
    MasterChannel,
    VirtualMasterChannel,
    SyncChannel,
    MaxLengthDataChannel,
    VirtualDataChannel,
}
impl ChannelType {
    fn new(channel_type: u8) -> Self {
        match channel_type {
            0 => Self::FixedLengthChannel,
            1 => Self::VariableLengthChannel,

            2 => Self::MasterChannel,
            3 => Self::VirtualMasterChannel,

            4 => Self::SyncChannel,
            5 => Self::MaxLengthDataChannel,

            6 => Self::VirtualDataChannel,
            _ => panic!("Error: Unknown channel type"),
        }
    }
}

#[derive(Debug, Clone)]
enum SyncType {
    None,
    Time,
    Angle,
    Distance,
    Index,
}
impl SyncType {
    fn new(channel_type: u8) -> Self {
        match channel_type {
            0 => Self::None,
            1 => Self::Time,
            2 => Self::Angle,
            3 => Self::Distance,
            4 => Self::Index,
            _ => panic!("Error: Unknown sync type"),
        }
    }
}

#[derive(Debug, Clone)]
enum DataType {
    UnsignedByteLE,
    UnsignedByteBE,
    SignedLE,
    SignedBE,
    FloatLE,
    FloatBE,
    StringLatin,
    StringUTF8,
    StringUTF16LE,
    StringUTF16BE,
    ByteArray,
    MIMESample,
    CANopenData,
    CANopenTime,
}
impl DataType {
    fn new(channel_type: u8) -> Self {
        match channel_type {
            0 => Self::UnsignedByteBE,
            1 => Self::UnsignedByteBE,
            2 => Self::SignedLE,
            3 => Self::SignedBE,
            4 => Self::FloatLE,
            5 => Self::FloatBE,
            6 => Self::StringLatin,
            7 => Self::StringUTF8,
            8 => Self::StringUTF16LE,
            9 => Self::StringUTF16BE,
            10 => Self::ByteArray,
            11 => Self::MIMESample,
            12 => Self::CANopenData,
            13 => Self::CANopenTime,
            _ => panic!("Error: Unknown data type"),
        }
    }
}

#[derive(Debug, Clone)]
enum CCType {
    Direct,
    Parametic,
    Rational,
    Algebraic,
    ValueTableInterpolate,
    ValueTableNoInterpolate,
    RangeTableValue,
    ValueTableText,
    RangeTableText,
    TextTableValue,
    TextTableText,
}
impl CCType {
    fn new(cc_type: u8) -> Self {
        match cc_type {
            0 => Self::Direct,
            1 => Self::Parametic,
            2 => Self::Rational,
            3 => Self::Algebraic,
            4 => Self::ValueTableInterpolate,

            5 => Self::ValueTableNoInterpolate,

            6 => Self::RangeTableValue,
            7 => Self::ValueTableText,
            8 => Self::RangeTableText,
            9 => Self::TextTableValue,
            10 => Self::TextTableText,
            _ => panic!("Error CCtype"),
        }
    }
}

#[derive(Debug, Clone)]
struct FileIdentificationBlock {}
impl Block for FileIdentificationBlock {
    fn new() -> Self {
        FileIdentificationBlock {}
    }
    fn default() -> Self {
        FileIdentificationBlock {}
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        (1, FileIdentificationBlock {})
    }
}

#[derive(Debug, Clone)]
struct ListData {}
impl Block for ListData {
    fn new() -> Self {
        ListData {}
    }
    fn default() -> Self {
        ListData {}
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        (1, ListData {})
    }
}
