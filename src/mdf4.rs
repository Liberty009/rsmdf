use crate::mdf::{self, MdfChannel};
use crate::record::Record;
use crate::{signal, utils};
use std::fs::File;
use std::io::prelude::*;
use std::{convert::TryInto, mem};


trait Block {
	fn new() -> Self;
	fn default() -> Self;
	fn read(stream: &[u8]) -> (Self, usize);
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

struct AttachmentBlock {
    id: [u8; 4],
    reserved0: u8,
    block_len: u8,
    links_nr: u8,
    next_at_addr: u8,
    file_name_addr: u8,
    mime_addr: u8,
    comment_addr: u8,
    flags: u8,
    creator_index: u8,
    reserved1: u8,
    md5_sum: [u8; 10],
    original_size: u8,
    embedded_size: u8,
    embedded_data: u8,
    address: u8,
    file_name: String,
    mime: String,
    comment: String,
}

impl Block for AttachmentBlock{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){

		let mut pos = 0;

		let id: [u8; 4] = stream[pos..pos+4].try_into().expect("msg");
		if !utils::eq(
			&id[..], 
		&[0x23,0x23,0x43,0x4E]) {
			panic!("Error: Reading Attachment block");
		}

	}
}

struct CNBLOCK {
    id: [u8; 4],        //block ID; always b'##CN'
    reserved0: u8,      //reserved bytes
    block_len: u8,      //block bytes size
    links_nr: u8,       //number of links
    next_ch_addr: u8,   //next ATBLOCK address
    component_addr: u8, //address of first channel in case of structure channel
    //   composition, or ChannelArrayBlock in case of arrays
    //   file name
    name_addr: u8,       //address of TXBLOCK that contains the channel name
    source_addr: u8,     //address of channel source block
    conversion_addr: u8, //address of channel conversion block
    data_block_addr: u8, //address of signal data block for VLSD channels
    unit_addr: u8,       //address of TXBLOCK that contains the channel unit
    comment_addr: u8,    //address of TXBLOCK/MDBLOCK that contains the
    //   channel comment
    attachment_addr: u8, //address of N:th ATBLOCK referenced by the
    //   current channel; if no ATBLOCK is referenced there will be no such key:value
    //   pair
    default_X_dg_addr: u8, //address of DGBLOCK where the default X axis
    //   channel for the current channel is found; this key:value pair will not
    //   exist for channels that don't have a default X axis
    default_X_cg_addr: u8, //address of CGBLOCK where the default X axis
    //   channel for the current channel is found; this key:value pair will not
    //   exist for channels that don't have a default X axis
    default_X_ch_addr: u8, //address of default X axis
    //   channel for the current channel; this key:value pair will not
    //   exist for channels that don't have a default X axis
    channel_type: u8,         //integer code for the channel type
    sync_type: u8,            //integer code for the channel's sync type
    data_type: u8,            //integer code for the channel's data type
    bit_offset: u8,           //bit offset
    byte_offset: u8,          //byte offset within the data record
    bit_count: u8,            //channel bit count
    flags: u8,                //CNBLOCK flags
    pos_invalidation_bit: u8, //invalidation bit position for the current
    //   channel if there are invalidation bytes in the data record
    precision: u8,       //integer code for the precision
    reserved1: u8,       //reserved bytes
    min_raw_value: u8,   //min raw value of all samples
    max_raw_value: u8,   //max raw value of all samples
    lower_limit: u8,     //min physical value of all samples
    upper_limit: u8,     //max physical value of all samples
    lower_ext_limit: u8, //min physical value of all samples
    upper_ext_limit: u8, //max physical value of all samples

    // Other attributes
    address: u8,       //channel address
    attachments: list, //list of referenced attachment blocks indexes;
    //   the index reference to the attachment block index
    comment: String,               // channel comment
    conversion: CCBLOCK, // channel conversion; None if the
    //   channel has no conversion
    display_name: String, // channel display name; this is extracted from the
    //   XML channel comment
    name: String,              //channel name
    source: SourceInformation, // channel source information; None if
    //   the channel has no source information
    unit: String, // channel unit
}

impl Block for CNBLOCK{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}

struct CABLOCK {}

impl Block for CABLOCK{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}

struct CGBLOCK {}

impl Block for CGBLOCK{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}

struct CCBLOCK {}
impl Block for CCBLOCK{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}
struct DataBlock {}
impl Block for DataBlock{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}
struct DataZippedBlock {}
impl Block for DataZippedBlock{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}
struct DataGroup {}
impl Block for DataGroup{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}

struct DataList {}
impl Block for DataList{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}

struct EventBlock {}
impl Block for EventBlock{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}

struct FileIdentificationBlock {}
impl Block for FileIdentificationBlock{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}

struct FileHistory {}
impl Block for FileHistory{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}

struct HeaderBlock {}
impl Block for HeaderBlock{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}

struct HeaderList {}
impl Block for HeaderList{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}

struct ListData {}
impl Block for ListData{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}

struct SourceInformation {}
impl Block for SourceInformation{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read() -> (Self, usize){}
}

struct TextBlock {}
impl Block for TextBlock{
	fn new() -> Self{}
	fn default() -> Self{}
	fn read(stream: &[u8]) -> (Self, usize){}
}

