use crate::mdf::{self, MdfChannel, RasterType};
use crate::record::Record;
use crate::signal::{self, Signal};
use crate::utils;
use std::fs::File;
use std::io::prelude::*;
use std::mem;

struct BlockHeader {
    id: [u8; 4],
    //reserved0: [u8; 4],
    length: u64,
    link_count: u64,
}

impl Block for BlockHeader {
    fn new() -> Self {
        Self {
            id: [0; 4],
            //reserved0: [0; 4],
            length: 0,
            link_count: 0,
        }
    }
    fn default() -> Self {
        Self {
            id: [0; 4],
            //reserved0: [0; 4],
            length: 0,
            link_count: 0,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let mut pos = position;
        let id: [u8; 4] = utils::read(stream, little_endian, &mut pos);
        let _reserved0: [u8; 4] = utils::read(stream, little_endian, &mut pos);

        let length = utils::read(stream, little_endian, &mut pos);
        let link_count = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                id,
                //reserved0,
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

    for _i in 0..no_links {
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
pub struct MDF4 {
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

        let (position, _id_block) = IDBLOCK::read(&self.file, 0, little_endian);
        let (_pos, hd_block) = HDBLOCK::read(&self.file, position, little_endian);

        let mut next_dg = hd_block.hd_dg_first;

        while next_dg != 0 {
            let (_pos, dg_block) = DGBLOCK::read(&self.file, next_dg as usize, little_endian);
            next_dg = dg_block.dg_dg_next;
            let mut next_cg = dg_block.dg_cg_first;

            dg.push(dg_block);

            while next_cg != 0 {
                let (_position, cg_block) =
                    CGBLOCK::read(&self.file, next_cg as usize, little_endian);
                next_cg = cg_block.cg_cg_next;
                let mut next_cn = cg_block.cg_cn_first;
                cg.push(cg_block.clone());

                println!("Channel Group: {}", cg_block.cg_md_comment);

                while next_cn != 0 {
                    let (_position, cn_block) =
                        CNBLOCK::read(&self.file, next_cn as usize, little_endian);
                    next_cn = cn_block.cn_cn_next;
                    ch.push(cn_block.clone());

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
        let channel_group = self.channel_groups[channel_grp]
            .clone()
            .channels(&self.file, self.little_endian);
        for (i, channel) in channel_group.iter().enumerate() {
            if matches!(channel.channel_type, ChannelType::Master) {
                //channel.channel_type == TIME_CHANNEL_TYPE {
                return Ok(i);
            }
        }

        Err("No time series found for the channel selected")
    }

    fn read_channel(&self, _datagroup: usize, _channel_grp: usize, _channel: usize) -> Vec<Record> {
        Vec::new()
    }

    #[must_use]
    fn new(filepath: &str) -> Self {
        let mut file = File::open(filepath).expect("Could not read file");
        let mut stream = Vec::new();
        let _ = file.read_to_end(&mut stream);

        let little_endian = true;
        let position = 0;

        let (pos, id) = IDBLOCK::read(&stream, position, little_endian);
        let (_pos, header) = HDBLOCK::read(&stream, pos, little_endian);
        let (_pos, comment) = TXBLOCK::read(&stream, header.hd_md_comment as usize, little_endian);
        let mut mdf = Self {
            id,
            header: header.clone(),
            comment,
            data_groups: DGBLOCK::read_all(&stream, header.hd_dg_first as usize, little_endian),
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
            let group1 = group.clone();
            let mut grp = group1.read_channel_groups(&self.file, self.little_endian);
            channel_groups.append(&mut grp);
        }

        let mut channels = Vec::new();
        for grp in &channel_groups {
            let grp1 = grp.clone();

            channels.append(&mut grp1.channels(&self.file, self.little_endian));
        }

        self.channel_groups = channel_groups;
        self.channels = channels;
    }

    fn list(&mut self) {
        let little_endian = true;
        let position = 0;

        let (position, _id_block) = IDBLOCK::read(&self.file, position, little_endian);
        let (_pos, hd_block) = HDBLOCK::read(&self.file, position, little_endian);
        //position += pos;

        let dg = DGBLOCK::read_all(&self.file, hd_block.hd_dg_first as usize, little_endian);
        self.data_groups = dg;
    }

    fn list_channels(&self) {
        let mut dg = Vec::new();
        let mut cg = Vec::new();
        let mut ch = Vec::new();

        let little_endian = true;
        let postion = 0;

        let (position, _id_block) = IDBLOCK::read(&self.file, postion, little_endian);
        let (_pos, hd_block) = HDBLOCK::read(&self.file, position, little_endian);
        //position += pos;

        let mut next_dg = hd_block.hd_dg_first; // .data_group_block;

        while next_dg != 0 {
            let (_pos, dg_block) = DGBLOCK::read(&self.file, next_dg as usize, little_endian);
            next_dg = dg_block.dg_dg_next; //  .next;
            let mut next_cg = dg_block.dg_cg_first; // .first;

            dg.push(dg_block);

            while next_cg != 0 {
                let (_pos, cg_block) = CGBLOCK::read(&self.file, next_cg as usize, little_endian);
                next_cg = cg_block.cg_cg_next; //.next;
                let mut next_cn = cg_block.cg_cn_first; //.first;
                cg.push(cg_block.clone());

                println!("Channel Group: {}", cg_block.cg_md_comment); //.comment);

                while next_cn != 0 {
                    let (_pos, cn_block) =
                        CNBLOCK::read(&self.file, next_cn as usize, little_endian);
                    next_cn = cn_block.cn_cn_next; //.next;

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

    fn cut(&self, start: f64, _end: f64, _include_ends: bool, time_from_zero: bool) {
        let _delta = if time_from_zero { start } else { 0.0 };
    }

    fn export(&self, _format: &str, _filename: &str) {}
    fn filter(&self, _channels: &str) {}
    #[must_use]
    fn resample(&self, _raster: RasterType, _version: &str, _time_from_zero: bool) -> Self {
        self.clone()
    }
}

#[derive(Debug, Clone)]
struct IDBLOCK {
    #[allow(dead_code)]
    id_file: [u8; 8],
    #[allow(dead_code)]
    id_vers: [u8; 8],
    #[allow(dead_code)]
    id_prog: [u8; 8],

    //id_reserved1: [u8; 4],
    #[allow(dead_code)]
    id_ver: u16,
    //id_reserved2: [u8; 34],
}
impl Block for IDBLOCK {
    fn new() -> Self {
        Self {
            id_file: [0; 8],
            id_vers: [0; 8],
            id_prog: [0; 8],
            //id_reserved1: [0; 4],
            id_ver: 0,
            //id_reserved2: [0; 34],
        }
    }
    fn default() -> Self {
        Self {
            id_file: [0; 8],
            id_vers: [0; 8],
            id_prog: [0; 8],
            //id_reserved1: [0; 4],
            id_ver: 0,
            //id_reserved2: [0; 34],
        }
    }
    fn read(stream: &[u8], _position: usize, _little_endian: bool) -> (usize, Self) {
        let mut pos = 0;
        let litte_endian = true;
        let id_file = utils::read(stream, _little_endian, &mut pos);
        let id_vers = utils::read(stream, litte_endian, &mut pos);
        let id_prog = utils::read(stream, litte_endian, &mut pos);
        let _id_reserved1: [u8; 4] = utils::read(stream, litte_endian, &mut pos);
        let id_ver = utils::read(stream, litte_endian, &mut pos);
        let _id_reserved2: [u8; 34] = utils::read(stream, litte_endian, &mut pos);

        (
            pos,
            Self {
                id_file,
                id_vers,
                id_prog,
                //id_reserved1,
                id_ver,
                //id_reserved2,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct HDBLOCK {
    #[allow(dead_code)]
    hd_dg_first: u64,
    #[allow(dead_code)]
    hd_fh_first: u64,
    #[allow(dead_code)]
    hd_ch_first: u64,
    #[allow(dead_code)]
    hd_at_first: u64,
    #[allow(dead_code)]
    hd_ev_first: u64,
    #[allow(dead_code)]
    hd_md_comment: u64,
    #[allow(dead_code)]
    hd_start_time_ns: u64,
    #[allow(dead_code)]
    hd_tz_offset_min: i16,
    #[allow(dead_code)]
    hd_dst_offset_min: i16,
    #[allow(dead_code)]
    hd_time_flags: u8,
    #[allow(dead_code)]
    hd_time_class: u8,
    #[allow(dead_code)]
    hd_flags: u8,
    //hd_reserved: u8,
    #[allow(dead_code)]
    hd_start_angle_rad: f64,
    #[allow(dead_code)]
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
            //hd_reserved: 0,
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
            //hd_reserved: 0,
            hd_start_angle_rad: 0.0,
            hd_start_distance_m: 0.0,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##HD".as_bytes()) {
            panic!("Error HDBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

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
        let _hd_reserved: u8 = utils::read(stream, little_endian, &mut pos);
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
                //hd_reserved,
                hd_start_angle_rad,
                hd_start_distance_m,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct MDBLOCK {
    #[allow(dead_code)]
    md_data: String,
}
impl Block for MDBLOCK {
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

        if !utils::eq(&header.id, "##MD".as_bytes()) {
            panic!("Error type incorrect");
        }

        let mut md_data_temp = "";
        unsafe {
            md_data_temp =
                str_from_u8_nul_utf8_unchecked(&stream[pos..(pos + header.length as usize - 10)]);
        }

        let md_data = md_data_temp.to_string();

        (pos + md_data.len(), Self { md_data })
    }
}

/// # Safety
///
/// This function should not be called before the horsemen are ready.
pub unsafe fn str_from_u8_nul_utf8_unchecked(utf8_src: &[u8]) -> &str {
    let nul_range_end = utf8_src
        .iter()
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

        if !utils::eq(&header.id, "##MD".as_bytes()) {
            panic!("Error type incorrect");
        }

        let mut tx_data_temp = "";
        unsafe {
            tx_data_temp =
                str_from_u8_nul_utf8_unchecked(&stream[pos..(pos + header.length as usize - 10)]);
        }

        let tx_data = tx_data_temp.to_string();

        (pos + header.length as usize, Self { tx_data })
    }
}

#[derive(Debug, Clone)]
struct FHBLOCK {
    #[allow(dead_code)]
    fh_fh_next: u64,
    #[allow(dead_code)]
    fh_md_comment: u64,
    #[allow(dead_code)]
    fh_time_ns: u64,
    #[allow(dead_code)]
    fh_tz_offset_min: i16,
    #[allow(dead_code)]
    fh_dst_offset_min: i16,
    #[allow(dead_code)]
    fh_time_flags: u8,
}
impl Block for FHBLOCK {
    fn new() -> Self {
        Self {
            fh_fh_next: 0_u64,
            fh_md_comment: 0_u64,
            fh_time_ns: 0_u64,
            fh_tz_offset_min: 0_i16,
            fh_dst_offset_min: 0_i16,
            fh_time_flags: 0_u8,
        }
    }
    fn default() -> Self {
        Self {
            fh_fh_next: 0_u64,
            fh_md_comment: 0_u64,
            fh_time_ns: 0_u64,
            fh_tz_offset_min: 0_i16,
            fh_dst_offset_min: 0_i16,
            fh_time_flags: 0_u8,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##FH".as_bytes()) {
            panic!("Error FHBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let fh_fh_next = address.remove(0);
        let fh_md_comment = address.remove(0);

        let fh_time_ns = utils::read(stream, little_endian, &mut pos);
        let fh_tz_offset_min = utils::read(stream, little_endian, &mut pos);
        let fh_dst_offset_min = utils::read(stream, little_endian, &mut pos);
        let fh_time_flags = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                fh_fh_next,
                fh_md_comment,
                fh_time_ns,
                fh_tz_offset_min,
                fh_dst_offset_min,
                fh_time_flags,
            },
        )
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
    fn new(ch_type: u8) -> Self {
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
            _ => panic!("Unknown channel type"),
        }
    }
}

struct CHBLOCK {
    #[allow(dead_code)]
    ch_ch_next: u64,
    #[allow(dead_code)]
    ch_ch_first: u64,
    #[allow(dead_code)]
    ch_tx_name: u64,
    #[allow(dead_code)]
    ch_md_comment: u64,
    #[allow(dead_code)]
    ch_element: Vec<u64>,
    #[allow(dead_code)]
    ch_element_count: u32,
    #[allow(dead_code)]
    ch_type: ChannelHierarchyType,
}
impl Block for CHBLOCK {
    fn new() -> Self {
        Self {
            ch_ch_next: 0_u64,
            ch_ch_first: 0_u64,
            ch_tx_name: 0_u64,
            ch_md_comment: 0_u64,
            ch_element: Vec::new(),
            ch_element_count: 0_u32,
            ch_type: ChannelHierarchyType::Function,
        }
    }
    fn default() -> Self {
        Self {
            ch_ch_next: 0_u64,
            ch_ch_first: 0_u64,
            ch_tx_name: 0_u64,
            ch_md_comment: 0_u64,
            ch_element: Vec::new(),
            ch_element_count: 0_u32,
            ch_type: ChannelHierarchyType::Function,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##CH".as_bytes()) {
            panic!("Error CHBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let ch_element_count = utils::read(stream, little_endian, &mut pos);
        let ch_type = ChannelHierarchyType::new(utils::read(stream, little_endian, &mut pos));

        let ch_ch_next = address.remove(0);
        let ch_ch_first = address.remove(0);
        let ch_tx_name = address.remove(0);
        let ch_md_comment = address.remove(0);
        let mut ch_element = Vec::with_capacity(ch_element_count as usize * 3);
        for i in 0..(ch_element_count * 3) {
            ch_element.push(address.remove(0));
        }

        (
            pos,
            Self {
                ch_ch_next,
                ch_ch_first,
                ch_tx_name,
                ch_md_comment,
                ch_element,
                ch_element_count,
                ch_type,
            },
        )
    }
}
#[derive(Debug, Clone)]
struct ATBLOCK {
    //id: [u8; 4],
    //reserved0: [u8; 4],
    //block_len: u64,
    //links_nr: u64,
    #[allow(dead_code)]
    next_at_addr: u64,
    #[allow(dead_code)]
    file_name_addr: u64,
    #[allow(dead_code)]
    mime_addr: u64,
    #[allow(dead_code)]
    comment_addr: u64,
    #[allow(dead_code)]
    flags: u16,
    #[allow(dead_code)]
    creator_index: u16,
    //reserved1: [u8; 4],
    #[allow(dead_code)]
    md5_sum: [u8; 16],
    #[allow(dead_code)]
    original_size: u64,
    #[allow(dead_code)]
    embedded_size: u64,
    #[allow(dead_code)]
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

        if !utils::eq(&header.id, "##AT".as_bytes()) {
            panic!("Error: block id doesn't match Attachment Block");
        }

        let (mut pos, addresses) = link_extract(stream, pos, little_endian, header.link_count);

        let next_at_addr = addresses[0];
        let file_name_addr = addresses[1];
        let mime_addr = addresses[2];
        let comment_addr = addresses[3];

        let flags = utils::read(stream, little_endian, &mut pos);
        let creator_index = utils::read(stream, little_endian, &mut pos);
        let _reserved1: [u8; 4] = utils::read(stream, little_endian, &mut pos);
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
    #[allow(dead_code)]
    ev_ev_next: u64,
    #[allow(dead_code)]
    ev_ev_parent: u64,
    #[allow(dead_code)]
    ev_ev_range: u64,
    #[allow(dead_code)]
    ev_tx_name: u64,
    #[allow(dead_code)]
    ev_md_comment: u64,
    #[allow(dead_code)]
    ev_scope: Vec<u64>,
    #[allow(dead_code)]
    ev_at_reference: Vec<u64>,
    #[allow(dead_code)]
    ev_type: EventType,
    #[allow(dead_code)]
    ev_sync_type: EventSyncType,
    #[allow(dead_code)]
    ev_range_type: RangeType,
    #[allow(dead_code)]
    ev_cause: EventCause,
    #[allow(dead_code)]
    ev_flags: u8,
    #[allow(dead_code)]
    ev_scope_count: u32,
    #[allow(dead_code)]
    ev_attachment_count: u16,
    #[allow(dead_code)]
    ev_creator_index: u16,
    #[allow(dead_code)]
    ev_sync_base_value: i64,
    #[allow(dead_code)]
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
            ev_creator_index: 0_u16,
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
            ev_creator_index: 0_u16,
            ev_sync_base_value: 0_i64,
            ev_sync_factor: 0_f64,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);
        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let ev_type = EventType::new(utils::read(stream, little_endian, &mut pos));
        let ev_sync_type = EventSyncType::new(utils::read(stream, little_endian, &mut pos));
        let ev_range_type = RangeType::new(utils::read(stream, little_endian, &mut pos));
        let ev_cause = EventCause::new(utils::read(stream, little_endian, &mut pos));
        let ev_flags = utils::read(stream, little_endian, &mut pos);

        let _ev_reserved: [u8; 3] = utils::read(stream, little_endian, &mut pos);

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
        for _i in 0..ev_scope_count {
            ev_scope.push(address.remove(0));
        }
        let mut ev_at_reference = Vec::new();
        for _i in 0..ev_attachment_count {
            ev_at_reference.push(address.remove(0));
        }

        (
            pos,
            Self {
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
            },
        )
    }
}

#[derive(Debug, Clone)]

enum EventType {
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
            _ => panic!("Error with Event Type"),
        }
    }
}

#[derive(Debug, Clone)]

enum EventSyncType {
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
            _ => panic!("Error Event Sync Type"),
        }
    }
}

#[derive(Debug, Clone)]

enum RangeType {
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
            _ => panic!("Error Range Type"),
        }
    }
}

#[derive(Debug, Clone)]
enum EventCause {
    Other,
    Error,
    Tool,
    Script,
    User,
}

impl EventCause {
    fn new(ev_cause: u8) -> Self {
        match ev_cause {
            0 => Self::Other,
            1 => Self::Error,
            2 => Self::Tool,
            3 => Self::Script,
            4 => Self::User,
            _ => panic!("Error Event cause"),
        }
    }
}

#[derive(Debug, Clone)]
struct DGBLOCK {
    #[allow(dead_code)]
    dg_dg_next: u64,
    #[allow(dead_code)]
    dg_cg_first: u64,
    #[allow(dead_code)]
    dg_data: u64,
    #[allow(dead_code)]
    dg_md_comment: u64,
    #[allow(dead_code)]
    dg_rec_id_size: u8,
}

impl DGBLOCK {
    fn read_all(stream: &[u8], position: usize, little_endian: bool) -> Vec<Self> {
        let mut all = Vec::new();
        let mut next_dg = position;

        while next_dg != 0 {
            let (_pos, dg_block) = DGBLOCK::read(stream, next_dg, little_endian);
            next_dg = dg_block.dg_dg_next as usize;
            all.push(dg_block);
        }

        all
    }

    fn read_channel_groups(self, stream: &[u8], little_endian: bool) -> Vec<CGBLOCK> {
        let mut channel_grps = Vec::new();
        let mut next = self.dg_cg_first as usize;
        while next != 0 {
            let (_pos, cg_block) = CGBLOCK::read(stream, next, little_endian);
            next = cg_block.cg_cg_next as usize;
            channel_grps.push(cg_block);
        }
        channel_grps
    }
}

impl Block for DGBLOCK {
    fn new() -> Self {
        Self {
            dg_dg_next: 0_u64,
            dg_cg_first: 0_u64,
            dg_data: 0_u64,
            dg_md_comment: 0_u64,
            dg_rec_id_size: 0_u8,
        }
    }
    fn default() -> Self {
        Self {
            dg_dg_next: 0_u64,
            dg_cg_first: 0_u64,
            dg_data: 0_u64,
            dg_md_comment: 0_u64,
            dg_rec_id_size: 0_u8,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);
        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let dg_rec_id_size = utils::read(stream, little_endian, &mut pos);
        let _dg_reserved: [u8; 7] = utils::read(stream, little_endian, &mut pos);

        let dg_dg_next = address.remove(0);
        let dg_cg_first = address.remove(0);
        let dg_data = address.remove(0);
        let dg_md_comment = address.remove(0);

        (
            pos,
            Self {
                dg_dg_next,
                dg_cg_first,
                dg_data,
                dg_md_comment,
                dg_rec_id_size,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct CGBLOCK {
    //id: [u8; 4],        //- bytes : block ID; always b'##CG'
    //reserved0: u64,     //- int : reserved bytes
    //block_len: u64,     //- int : block bytes size
    //links_nr: u64,      //- int : number of links
    #[allow(dead_code)]
    cg_cg_next: u64, //- int : next channel group address
    #[allow(dead_code)]
    cg_cn_first: u64, //- int : address of first channel of this channel group
    #[allow(dead_code)]
    cg_tx_acq_name: u64, //- int : address of TextBLock that contains the channel
    #[allow(dead_code)]
    cg_si_acq_source: u64, //- int : address of SourceInformation that contains the
    #[allow(dead_code)]
    cg_sr_first: u64, // - int : address of first SRBLOCK; this is
    #[allow(dead_code)]
    cg_md_comment: u64, //- int : address of TXBLOCK/MDBLOCK that contains the
    #[allow(dead_code)]
    cg_record_id: u64, //- int : record ID for the channel group
    #[allow(dead_code)]
    cg_cycle_count: u64, //- int : number of cycles for this channel group
    #[allow(dead_code)]
    cg_flags: u64, //- int : channel group flags
    #[allow(dead_code)]
    cg_path_separator: u8,
    #[allow(dead_code)]
    cg_data_bytes: u64,
    #[allow(dead_code)]
    cg_inval_bytes: u64, // - int : number of bytes used for invalidation
                         // bits by this channel group

                         //Other attributes
                         //acq_name: u64,   // - str : acquisition name
                         //acq_source: u64, //- SourceInformation : acquisition source information
                         //address: u64,    //- int : channel group address
                         //comment: u64,    //- str : channel group comment
}

impl CGBLOCK {
    fn channels(self, stream: &[u8], little_endian: bool) -> Vec<CNBLOCK> {
        let mut ch = Vec::new();
        let mut next_cn = self.cg_cn_first as usize;
        while next_cn != 0 {
            let (_pos, cn_block) = CNBLOCK::read(stream, next_cn, little_endian);
            next_cn = cn_block.cn_cn_next as usize;

            ch.push(cn_block);
        }

        ch
    }
}

impl Block for CGBLOCK {
    fn new() -> Self {
        CGBLOCK {
            cg_cg_next: 0,
            cg_cn_first: 0,
            cg_tx_acq_name: 0,
            cg_si_acq_source: 0,
            cg_sr_first: 0,
            cg_md_comment: 0,
            cg_record_id: 0,
            cg_cycle_count: 0,
            cg_flags: 0,
            cg_path_separator: 0,
            cg_data_bytes: 0,
            cg_inval_bytes: 0,
        }
    }
    fn default() -> Self {
        CGBLOCK {
            cg_cg_next: 0,
            cg_cn_first: 0,
            cg_tx_acq_name: 0,
            cg_si_acq_source: 0,
            cg_sr_first: 0,
            cg_md_comment: 0,
            cg_record_id: 0,
            cg_cycle_count: 0,
            cg_flags: 0,
            cg_path_separator: 0,
            cg_data_bytes: 0,
            cg_inval_bytes: 0,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##CG".as_bytes()) {
            panic!("Error: Channel group wrong id");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let next_cg_addr = address.remove(0);
        let first_ch_addr = address.remove(0);
        let acq_name_addr = address.remove(0);
        let acq_source_addr = address.remove(0);
        let first_sample_reduction_addr = address.remove(0);
        let comment_addr = address.remove(0);

        let record_id = utils::read(stream, little_endian, &mut pos);
        let cycles_nr = utils::read(stream, little_endian, &mut pos);
        let flags = utils::read(stream, little_endian, &mut pos);
        let path_separator = utils::read(stream, little_endian, &mut pos);
        let _reserved1: [u8; 4] = utils::read(stream, little_endian, &mut pos);
        let samples_byte_nr = utils::read(stream, little_endian, &mut pos);
        let invalidation_bytes_nr = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            CGBLOCK {
                // id,
                // reserved0,
                // block_len,
                // links_nr,
                cg_cg_next: next_cg_addr,
                cg_cn_first: first_ch_addr,
                cg_tx_acq_name: acq_name_addr,
                cg_si_acq_source: acq_source_addr,
                cg_sr_first: first_sample_reduction_addr,
                cg_md_comment: comment_addr,
                cg_record_id: record_id,
                cg_cycle_count: cycles_nr,
                cg_flags: flags,
                cg_path_separator: path_separator,
                //reserved1,
                cg_data_bytes: samples_byte_nr,
                cg_inval_bytes: invalidation_bytes_nr,
                // acq_name,
                // comment,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct SIBLOCK {
    #[allow(dead_code)]
    si_tx_name: u64,
    #[allow(dead_code)]
    si_tx_path: u64,
    #[allow(dead_code)]
    si_md_comment: u64,
    #[allow(dead_code)]
    si_type: SourceType,
    #[allow(dead_code)]
    si_bus_type: BusType,
    #[allow(dead_code)]
    si_flags: u8,
}
impl Block for SIBLOCK {
    fn new() -> Self {
        SIBLOCK {
            si_tx_name: 0_u64,
            si_tx_path: 0_u64,
            si_md_comment: 0_u64,
            si_type: SourceType::Bus,
            si_bus_type: BusType::Can,
            si_flags: 0_u8,
        }
    }
    fn default() -> Self {
        SIBLOCK {
            si_tx_name: 0_u64,
            si_tx_path: 0_u64,
            si_md_comment: 0_u64,
            si_type: SourceType::Bus,
            si_bus_type: BusType::Can,
            si_flags: 0_u8,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##SI".as_bytes()) {
            panic!("Error SIBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let si_tx_name = address.remove(0);
        let si_tx_path = address.remove(0);
        let si_md_comment = address.remove(0);

        let si_type = SourceType::new(utils::read(stream, little_endian, &mut pos));
        let si_bus_type = BusType::new(utils::read(stream, little_endian, &mut pos));
        let si_flags = utils::read(stream, little_endian, &mut pos);

        let _si_reserved: [u8; 5] = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            SIBLOCK {
                si_tx_name,
                si_tx_path,
                si_md_comment,
                si_type,
                si_bus_type,
                si_flags,
            },
        )
    }
}

#[derive(Debug, Clone)]
enum SourceType {
    Other,
    ECU,
    Bus,
    IO,
    Tool,
    User,
}

impl SourceType {
    fn new(source: u8) -> Self {
        match source {
            0 => Self::Other,
            1 => Self::ECU,
            2 => Self::Bus,
            3 => Self::IO,
            4 => Self::Tool,
            5 => Self::User,
            _ => panic!("Error source type"),
        }
    }
}

#[derive(Debug, Clone)]
enum BusType {
    None,
    Other,
    Can,
    Lin,
    Most,
    FlexRay,
    KLine,
    Ethernet,
    Usb,
}

impl BusType {
    fn new(source: u8) -> Self {
        match source {
            0 => Self::None,
            1 => Self::Other,
            2 => Self::Can,
            3 => Self::Lin,
            4 => Self::Most,
            5 => Self::FlexRay,
            6 => Self::KLine,
            7 => Self::Ethernet,
            8 => Self::Usb,
            _ => panic!("Error bus type"),
        }
    }
}

#[derive(Debug, Clone)]
struct CNBLOCK {
    //id: [u8; 4],        //block ID; always b'##CN'
    //reserved0: u32,      //reserved bytes
    //block_len: u64,      //block bytes size
    //links_nr: u64,       //number of links
    #[allow(dead_code)]
    cn_cn_next: u64, //next ATBLOCK address
    #[allow(dead_code)]
    cn_composition: u64,
    #[allow(dead_code)]
    cn_tx_name: u64, //address of TXBLOCK that contains the channel name
    #[allow(dead_code)]
    cn_si_source: u64, //address of channel source block
    #[allow(dead_code)]
    cn_cc_conversion: u64, //address of channel conversion block
    #[allow(dead_code)]
    cn_data: u64, //address of signal data block for VLSD channels
    #[allow(dead_code)]
    cn_md_unit: u64, //address of TXBLOCK that contains the channel unit
    #[allow(dead_code)]
    cn_md_comment: u64,
    #[allow(dead_code)]
    cn_at_reference: Vec<u64>,
    #[allow(dead_code)]
    cn_default_x: Vec<u64>,
    #[allow(dead_code)]
    channel_type: ChannelType, //integer code for the channel type
    #[allow(dead_code)]
    sync_type: SyncType, //integer code for the channel's sync type
    #[allow(dead_code)]
    data_type: DataType, //integer code for the channel's data type
    #[allow(dead_code)]
    bit_offset: u8, //bit offset
    #[allow(dead_code)]
    byte_offset: u32, //byte offset within the data record
    #[allow(dead_code)]
    bit_count: u32, //channel bit count
    #[allow(dead_code)]
    flags: u32, //CNBLOCK flags
    #[allow(dead_code)]
    pos_invalidation_bit: u32, //invalidation bit position for the current
    #[allow(dead_code)]
    precision: u8, //integer code for the precision
    #[allow(dead_code)]
    min_raw_value: f64, //min raw value of all samples
    #[allow(dead_code)]
    max_raw_value: f64, //max raw value of all samples
    #[allow(dead_code)]
    lower_limit: f64, //min physical value of all samples
    #[allow(dead_code)]
    upper_limit: f64, //max physical value of all samples
    #[allow(dead_code)]
    lower_ext_limit: f64, //min physical value of all samples
    #[allow(dead_code)]
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

impl CNBLOCK {
    fn name(self, stream: &[u8], little_endian: bool) -> String {
        let mut name = "".to_string();

        if matches!(self.channel_type, ChannelType::Master) {
            name = "time".to_string();
        } else if self.cn_tx_name != 0 {
            let (_pos, tx) = TXBLOCK::read(stream, self.cn_tx_name as usize, little_endian);

            name = tx.tx_data;
        }

        name
    }
}

impl Block for CNBLOCK {
    fn new() -> Self {
        CNBLOCK {
            cn_cn_next: 0,
            cn_composition: 0,
            cn_tx_name: 0,
            cn_si_source: 0,
            cn_cc_conversion: 0,
            cn_data: 0,
            cn_md_unit: 0,
            cn_md_comment: 0,
            cn_at_reference: Vec::new(),
            cn_default_x: Vec::new(),
            channel_type: ChannelType::FixedLength,
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
            cn_cn_next: 0,
            cn_composition: 0,
            cn_tx_name: 0,
            cn_si_source: 0,
            cn_cc_conversion: 0,
            cn_data: 0,
            cn_md_unit: 0,
            cn_md_comment: 0,
            cn_at_reference: Vec::new(),
            cn_default_x: Vec::new(),
            channel_type: ChannelType::FixedLength,
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

        if !utils::eq(&header.id, "##CN".as_bytes()) {
            panic!("Error: Incorrect channel id");
        }

        let (mut pos, mut addresses) = link_extract(stream, pos, little_endian, header.link_count);

        let channel_type = ChannelType::new(utils::read(stream, little_endian, &mut pos));
        let sync_type = SyncType::new(utils::read(stream, little_endian, &mut pos));
        let data_type = DataType::new(utils::read(stream, little_endian, &mut pos));

        let bit_offset = utils::read(stream, little_endian, &mut pos);
        let byte_offset = utils::read(stream, little_endian, &mut pos);
        let bit_count = utils::read(stream, little_endian, &mut pos);
        let flags = utils::read(stream, little_endian, &mut pos);
        let invalidation_bit_pos = utils::read(stream, little_endian, &mut pos);
        let precision = utils::read(stream, little_endian, &mut pos);
        let _reserved1: u8 = utils::read(stream, little_endian, &mut pos);
        let attachment_nr: u16 = utils::read(stream, little_endian, &mut pos);
        let min_raw_value = utils::read(stream, little_endian, &mut pos);
        let max_raw_value = utils::read(stream, little_endian, &mut pos);
        let lower_limit = utils::read(stream, little_endian, &mut pos);
        let upper_limit = utils::read(stream, little_endian, &mut pos);
        let lower_ext_limit = utils::read(stream, little_endian, &mut pos);
        let upper_ext_limit = utils::read(stream, little_endian, &mut pos);

        let cn_cn_next = addresses.remove(0);
        let cn_composition = addresses.remove(0);
        let cn_tx_name = addresses.remove(0);
        let cn_si_source = addresses.remove(0);
        let cn_cc_conversion = addresses.remove(0);
        let cn_data = addresses.remove(0);
        let cn_md_unit = addresses.remove(0);
        let cn_md_comment = addresses.remove(0);

        let mut cn_at_reference = Vec::with_capacity(attachment_nr as usize);
        for _i in 0..attachment_nr {
            cn_at_reference.push(addresses.remove(0));
        }

        let mut cn_default_x = Vec::with_capacity(3);
        for _i in 0..3 {
            cn_default_x.push(addresses.remove(0));
        }

        (
            1,
            CNBLOCK {
                cn_cn_next,
                cn_composition,
                cn_tx_name,
                cn_si_source,
                cn_cc_conversion,
                cn_data,
                cn_md_unit,
                cn_md_comment,
                cn_at_reference,
                cn_default_x,
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
    #[allow(dead_code)]
    name_addr: u64,
    #[allow(dead_code)]
    unit_addr: u64,
    #[allow(dead_code)]
    comment_addr: u64,
    #[allow(dead_code)]
    inv_conv_addr: u64,
    #[allow(dead_code)]
    cc_ref: Vec<u64>,
    #[allow(dead_code)]
    conversion_type: CCType,
    #[allow(dead_code)]
    precision: u8,
    #[allow(dead_code)]
    flags: u16,
    #[allow(dead_code)]
    ref_param_nr: u16,
    #[allow(dead_code)]
    val_param_nr: u16,
    #[allow(dead_code)]
    min_phy_value: f64,
    #[allow(dead_code)]
    max_phy_value: f64,
    #[allow(dead_code)]
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
        let (mut pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id[..], "##CC".as_bytes()) {
            panic!("Error: id incorrect");
        }

        let name_addr = utils::read(stream, little_endian, &mut pos);
        let unit_addr = utils::read(stream, little_endian, &mut pos);
        let comment_addr = utils::read(stream, little_endian, &mut pos);
        let inv_conv_addr = utils::read(stream, little_endian, &mut pos);

        let cc_ref_length = (header.link_count - 4) as usize;
        let mut cc_ref = Vec::new();

        for _i in 0..cc_ref_length {
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
        for _i in 0..val_param_nr {
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
    #[allow(dead_code)]
    ca_composition: u64,
    #[allow(dead_code)]
    ca_data: Vec<u64>,
    #[allow(dead_code)]
    ca_dynamic_size: Vec<u64>,
    #[allow(dead_code)]
    ca_input_quantity: Vec<u64>,
    #[allow(dead_code)]
    ca_output_quantity: Vec<u64>,
    #[allow(dead_code)]
    ca_comparison_quantity: Vec<u64>,
    #[allow(dead_code)]
    ca_cc_axis_conversion: Vec<u64>,
    #[allow(dead_code)]
    ca_axis: Vec<u64>,
    #[allow(dead_code)]
    ca_type: u8,
    #[allow(dead_code)]
    ca_storage: u8,
    #[allow(dead_code)]
    ca_ndim: u16,
    #[allow(dead_code)]
    ca_flags: u32,
    #[allow(dead_code)]
    ca_byte_offset_base: i32,
    #[allow(dead_code)]
    ca_inval_bit_pos_base: u32,
    #[allow(dead_code)]
    ca_dim_size: Vec<u64>,
    #[allow(dead_code)]
    ca_axis_value: Vec<f64>,
    #[allow(dead_code)]
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
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id[..], "##CA".as_bytes()) {
            panic!("Error: id CABLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let ca_type = utils::read(stream, little_endian, &mut pos);
        let ca_storage = utils::read(stream, little_endian, &mut pos);
        let ca_ndim = utils::read(stream, little_endian, &mut pos);
        let ca_flags = utils::read(stream, little_endian, &mut pos);
        let ca_byte_offset_base = utils::read(stream, little_endian, &mut pos);
        let ca_inval_bit_pos_base = utils::read(stream, little_endian, &mut pos);

        let d = ca_ndim as usize;

        let mut ca_dim_size = Vec::new();
        for _i in 0..d {
            ca_dim_size.push(utils::read(stream, little_endian, &mut pos));
        }

        let nd_sum = ca_dim_size.iter().sum();
        let nd_prod = ca_dim_size.iter().product();

        let mut ca_axis_value = Vec::new();
        for _i in 0..nd_sum {
            ca_axis_value.push(utils::read(stream, little_endian, &mut pos));
        }

        let mut ca_cycle_count = Vec::new();
        for _i in 0..nd_prod {
            ca_cycle_count.push(utils::read(stream, little_endian, &mut pos));
        }

        let ca_composition = address.remove(0);
        let mut ca_data = Vec::new();
        for _i in 0..nd_prod {
            ca_data.push(address.remove(0));
        }
        let mut ca_dynamic_size = Vec::new();
        for _i in 0..(3 * d) {
            ca_dynamic_size.push(address.remove(0));
        }
        let mut ca_input_quantity = Vec::new();
        for _i in 0..(3 * d) {
            ca_input_quantity.push(address.remove(0));
        }
        let mut ca_output_quantity = Vec::new();
        for _i in 0..3 {
            ca_output_quantity.push(address.remove(0));
        }
        let mut ca_comparison_quantity = Vec::new();
        for _i in 0..3 {
            ca_comparison_quantity.push(address.remove(0));
        }
        let mut ca_cc_axis_conversion = Vec::new();
        for _i in 0..d {
            ca_cc_axis_conversion.push(address.remove(0));
        }
        let mut ca_axis = Vec::new();
        for _i in 0..(3 * d) {
            ca_axis.push(address.remove(0));
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

// #[derive(Debug, Clone)]
// struct DTBlock {
// 	dt_data: Vec<Record>
// }
// impl Block for DTBlock {
//     fn new() -> Self {
//         Self {}
//     }
//     fn default() -> Self {
//         Self {}
//     }
//     fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
//         (1, Self {})
//     }
// }

#[derive(Debug, Clone)]
struct SRBLOCK {
    #[allow(dead_code)]
    sr_sr_next: u64,
    #[allow(dead_code)]
    sr_data: u64,
    #[allow(dead_code)]
    sr_cycle_count: u64,
    #[allow(dead_code)]
    sr_interval: f64,
    #[allow(dead_code)]
    sr_sync_type: u8,
    #[allow(dead_code)]
    sr_flags: u8,
}

impl Block for SRBLOCK {
    fn new() -> Self {
        Self {
            sr_sr_next: 0_u64,
            sr_data: 0_u64,
            sr_cycle_count: 0_u64,
            sr_interval: 0_f64,
            sr_sync_type: 0_u8,
            sr_flags: 0_u8,
        }
    }
    fn default() -> Self {
        Self {
            sr_sr_next: 0_u64,
            sr_data: 0_u64,
            sr_cycle_count: 0_u64,
            sr_interval: 0_f64,
            sr_sync_type: 0_u8,
            sr_flags: 0_u8,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##SR".as_bytes()) {
            panic!("Error SRBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let sr_sr_next = address.remove(0);
        let sr_data = address.remove(0);

        let sr_cycle_count = utils::read(stream, little_endian, &mut pos);
        let sr_interval = utils::read(stream, little_endian, &mut pos);
        let sr_sync_type = utils::read(stream, little_endian, &mut pos);
        let sr_flags = utils::read(stream, little_endian, &mut pos);
        let _sr_reserved: [u8; 6] = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                sr_sr_next,
                sr_data,
                sr_cycle_count,
                sr_interval,
                sr_sync_type,
                sr_flags,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct RDBLOCK {}

#[derive(Debug, Clone)]
struct SDBLOCK {}

#[derive(Debug, Clone)]
struct DLBLOCK {
    #[allow(dead_code)]
    dl_dl_next: u64,
    #[allow(dead_code)]
    dl_data: Vec<u64>,
    #[allow(dead_code)]
    dl_flags: u8,
    #[allow(dead_code)]
    dl_count: u32,
    #[allow(dead_code)]
    dl_equal_length: u64,
    #[allow(dead_code)]
    dl_offset: Vec<u64>,
}
impl Block for DLBLOCK {
    fn new() -> Self {
        Self {
            dl_dl_next: 0_u64,
            dl_data: Vec::new(),
            dl_flags: 0_u8,
            dl_count: 0_u32,
            dl_equal_length: 0_u64,
            dl_offset: Vec::new(),
        }
    }
    fn default() -> Self {
        Self {
            dl_dl_next: 0_u64,
            dl_data: Vec::new(),
            dl_flags: 0_u8,
            dl_count: 0_u32,
            dl_equal_length: 0_u64,
            dl_offset: Vec::new(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##DL".as_bytes()) {
            panic!("Error DBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let dl_flags = utils::read(stream, little_endian, &mut pos);
        let dl_count = utils::read(stream, little_endian, &mut pos);
        let dl_equal_length = utils::read(stream, little_endian, &mut pos);
        let mut dl_offset = Vec::new();
        for _i in 0..dl_count {
            dl_offset.push(utils::read(stream, little_endian, &mut pos));
        }

        let dl_dl_next = address.remove(0);
        let mut dl_data = Vec::new();
        for _i in 0..dl_count {
            dl_data.push(address.remove(0));
        }

        (
            pos,
            Self {
                dl_dl_next,
                dl_data,
                dl_flags,
                dl_count,
                dl_equal_length,
                dl_offset,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct DZBlock {
    #[allow(dead_code)]
    dz_org_block_type: [u8; 2],
    #[allow(dead_code)]
    dz_zip_type: ZipType,
    //dz_reserved: u8,
    #[allow(dead_code)]
    dz_zip_parameter: u32,
    #[allow(dead_code)]
    dz_org_data_length: u64,
    #[allow(dead_code)]
    dz_data_length: u64,
    #[allow(dead_code)]
    dz_data: Vec<u8>,
}
impl Block for DZBlock {
    fn new() -> Self {
        Self {
            dz_org_block_type: [0_u8; 2],
            dz_zip_type: ZipType::Deflate,
            //dz_reserved: 0_u8,
            dz_zip_parameter: 0_u32,
            dz_org_data_length: 0_u64,
            dz_data_length: 0_u64,
            dz_data: Vec::new(),
        }
    }
    fn default() -> Self {
        Self {
            dz_org_block_type: [0_u8; 2],
            dz_zip_type: ZipType::Deflate,
            //dz_reserved: 0_u8,
            dz_zip_parameter: 0_u32,
            dz_org_data_length: 0_u64,
            dz_data_length: 0_u64,
            dz_data: Vec::new(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (mut pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##DZ".as_bytes()) {
            panic!("Error DZBLOCK");
        }

        let dz_org_block_type = utils::read(stream, little_endian, &mut pos);
        let dz_zip_type = ZipType::new(utils::read(stream, little_endian, &mut pos));
        let _dz_reserved: u8 = utils::read(stream, little_endian, &mut pos);
        let dz_zip_parameter = utils::read(stream, little_endian, &mut pos);
        let dz_org_data_length = utils::read(stream, little_endian, &mut pos);
        let dz_data_length = utils::read(stream, little_endian, &mut pos);
        let dz_data = stream[pos..pos + dz_data_length as usize].to_vec();

        pos += dz_data.len();

        (
            pos,
            Self {
                dz_org_block_type,
                dz_zip_type,
                //dz_reserved,
                dz_zip_parameter,
                dz_org_data_length,
                dz_data_length,
                dz_data,
            },
        )
    }
}

#[derive(Debug, Clone)]
enum ZipType {
    Deflate,
    TransposeDeflate,
}

impl ZipType {
    fn new(zip: u8) -> Self {
        match zip {
            0 => Self::Deflate,
            1 => Self::TransposeDeflate,
            _ => panic!("Error zip type"),
        }
    }
}

#[derive(Debug, Clone)]
struct HLBLOCK {
    #[allow(dead_code)]
    hl_dl_first: u64,
    #[allow(dead_code)]
    hl_flags: u16,
    #[allow(dead_code)]
    hl_zip_type: ZipType,
    //hl_reserved: [u8; 5],
}
impl Block for HLBLOCK {
    fn new() -> Self {
        Self {
            hl_dl_first: 0_u64,
            hl_flags: 0_u16,
            hl_zip_type: ZipType::Deflate,
            //hl_reserved: [0_u8; 5]
        }
    }
    fn default() -> Self {
        Self {
            hl_dl_first: 0_u64,
            hl_flags: 0_u16,
            hl_zip_type: ZipType::Deflate,
            //hl_reserved: [0_u8; 5]
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##HL".as_bytes()) {
            panic!("Error HLBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let hl_dl_first = address.remove(0);
        let hl_flags = utils::read(stream, little_endian, &mut pos);
        let hl_zip_type = ZipType::new(utils::read(stream, little_endian, &mut pos));
        let _hl_reserved: [u8; 5] = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                hl_dl_first,
                hl_flags,
                hl_zip_type,
                //hl_reserved,
            },
        )
    }
}

#[derive(Debug, Clone)]
enum ChannelType {
    FixedLength,
    VariableLength,
    Master,
    VirtualMaster,
    Sync,
    MaxLengthData,
    VirtualData,
}
impl ChannelType {
    fn new(channel_type: u8) -> Self {
        match channel_type {
            0 => Self::FixedLength,
            1 => Self::VariableLength,
            2 => Self::Master,
            3 => Self::VirtualMaster,
            4 => Self::Sync,
            5 => Self::MaxLengthData,
            6 => Self::VirtualData,
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
            0 => Self::UnsignedByteLE,
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
    fn len(&self) -> usize {
        match self {
            Self::UnsignedByteLE => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::UnsignedByteBE => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::SignedLE => mem::size_of::<i8>() / mem::size_of::<u8>(),
            Self::SignedBE => mem::size_of::<i8>() / mem::size_of::<u8>(),
            Self::FloatLE => mem::size_of::<f64>() / mem::size_of::<u8>(),
            Self::FloatBE => mem::size_of::<f64>() / mem::size_of::<u8>(),
            Self::StringLatin => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::StringUTF8 => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::StringUTF16LE => mem::size_of::<u16>() / mem::size_of::<u8>(),
            Self::StringUTF16BE => mem::size_of::<u16>() / mem::size_of::<u8>(),
            Self::ByteArray => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::MIMESample => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::CANopenData => mem::size_of::<u8>() / mem::size_of::<u8>(),
            Self::CANopenTime => mem::size_of::<u8>() / mem::size_of::<u8>(),
            // _ => panic!("")
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
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
