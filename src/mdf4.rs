use crate::mdf::{self, MdfChannel};
use crate::mdf3::Text;
use crate::record::Record;
use crate::{signal, utils};
use std::fs::File;
use std::io::prelude::*;
use std::{convert::TryInto, mem};

trait Block {
    fn new() -> Self;
    fn default() -> Self;
    fn read(stream: &[u8]) -> (usize, Self);
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

impl Block for AttachmentBlock {
    fn new() -> Self {}
    fn default() -> Self {}
    fn read(stream: &[u8]) -> (usize, Self) {
        let mut pos = 0;

        let id: [u8; 4] = stream[pos..pos + 4].try_into().expect("msg");
        if !utils::eq(&id[..], &[0x23, 0x23, 0x43, 0x4E]) {
            panic!("Error: Reading Attachment block");
        }
    }
}

#[derive(Debug, Clone)]
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
    comment: String,     // channel comment
    conversion: CCBLOCK, // channel conversion; None if the
    //   channel has no conversion
    display_name: String, // channel display name; this is extracted from the
    //   XML channel comment
    name: String,              //channel name
    source: SourceInformation, // channel source information; None if
    //   the channel has no source information
    unit: String, // channel unit
}

impl Block for CNBLOCK {
    fn new() -> Self {
        CNBLOCK {
            id: (),
            reserved0: (),
            block_len: (),
            links_nr: (),
            next_ch_addr: (),
            component_addr: (),
            name_addr: (),
            source_addr: (),
            conversion_addr: (),
            data_block_addr: (),
            unit_addr: (),
            comment_addr: (),
            attachment_addr: (),
            default_X_dg_addr: (),
            default_X_cg_addr: (),
            default_X_ch_addr: (),
            channel_type: (),
            sync_type: (),
            data_type: (),
            bit_offset: (),
            byte_offset: (),
            bit_count: (),
            flags: (),
            pos_invalidation_bit: (),
            precision: (),
            reserved1: (),
            min_raw_value: (),
            max_raw_value: (),
            lower_limit: (),
            upper_limit: (),
            lower_ext_limit: (),
            upper_ext_limit: (),
            address: (),
            attachments: (),
            comment: (),
            conversion: (),
            display_name: (),
            name: (),
            source: (),
            unit: (),
        }
    }
    fn default() -> Self {
        CNBLOCK {
            id: (),
            reserved0: (),
            block_len: (),
            links_nr: (),
            next_ch_addr: (),
            component_addr: (),
            name_addr: (),
            source_addr: (),
            conversion_addr: (),
            data_block_addr: (),
            unit_addr: (),
            comment_addr: (),
            attachment_addr: (),
            default_X_dg_addr: (),
            default_X_cg_addr: (),
            default_X_ch_addr: (),
            channel_type: (),
            sync_type: (),
            data_type: (),
            bit_offset: (),
            byte_offset: (),
            bit_count: (),
            flags: (),
            pos_invalidation_bit: (),
            precision: (),
            reserved1: (),
            min_raw_value: (),
            max_raw_value: (),
            lower_limit: (),
            upper_limit: (),
            lower_ext_limit: (),
            upper_ext_limit: (),
            address: (),
            attachments: (),
            comment: (),
            conversion: (),
            display_name: (),
            name: (),
            source: (),
            unit: (),
        }
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (
            1,
            CNBLOCK {
                id: (),
                reserved0: (),
                block_len: (),
                links_nr: (),
                next_ch_addr: (),
                component_addr: (),
                name_addr: (),
                source_addr: (),
                conversion_addr: (),
                data_block_addr: (),
                unit_addr: (),
                comment_addr: (),
                attachment_addr: (),
                default_X_dg_addr: (),
                default_X_cg_addr: (),
                default_X_ch_addr: (),
                channel_type: (),
                sync_type: (),
                data_type: (),
                bit_offset: (),
                byte_offset: (),
                bit_count: (),
                flags: (),
                pos_invalidation_bit: (),
                precision: (),
                reserved1: (),
                min_raw_value: (),
                max_raw_value: (),
                lower_limit: (),
                upper_limit: (),
                lower_ext_limit: (),
                upper_ext_limit: (),
                address: (),
                attachments: (),
                comment: (),
                conversion: (),
                display_name: (),
                name: (),
                source: (),
                unit: (),
            },
        )
    }
}

#[derive(Debug, Clone)]
struct CABLOCK {}

impl Block for CABLOCK {
    fn new() -> Self {
        CABLOCK {}
    }
    fn default() -> Self {
        CABLOCK {}
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, CABLOCK {})
    }
}

#[derive(Debug, Clone)]
struct CGBLOCK {}

impl Block for CGBLOCK {
    fn new() -> Self {
        CGBLOCK {}
    }
    fn default() -> Self {
        CGBLOCK {}
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, CGBLOCK {})
    }
}

#[derive(Debug, Clone)]
struct CCBLOCK {}
impl Block for CCBLOCK {
    fn new() -> Self {
        CCBLOCK {}
    }
    fn default() -> Self {
        CCBLOCK {}
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, CCBLOCK {})
    }
}

#[derive(Debug, Clone)]
struct DataBlock {}
impl Block for DataBlock {
    fn new() -> Self {
        DataBlock {}
    }
    fn default() -> Self {
        DataBlock {}
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, DataBlock {})
    }
}

#[derive(Debug, Clone)]
struct DataZippedBlock {}
impl Block for DataZippedBlock {
    fn new() -> Self {
        DataZippedBlock {}
    }
    fn default() -> Self {
        DataZippedBlock {}
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, DataZippedBlock {})
    }
}

#[derive(Debug, Clone)]
struct DataGroup {}
impl Block for DataGroup {
    fn new() -> Self {
        DataGroup {}
    }
    fn default() -> Self {
        DataGroup {}
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, DataGroup {})
    }
}

#[derive(Debug, Clone)]
struct DataList {}
impl Block for DataList {
    fn new() -> Self {
        DataList {}
    }
    fn default() -> Self {
        DataList {}
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, DataList {})
    }
}

#[derive(Debug, Clone)]
struct EventBlock {}
impl Block for EventBlock {
    fn new() -> Self {
        EventBlock {}
    }
    fn default() -> Self {
        EventBlock {}
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, EventBlock {})
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
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, FileIdentificationBlock {})
    }
}

#[derive(Debug, Clone)]
struct FileHistory {}
impl Block for FileHistory {
    fn new() -> Self {
        FileHistory {}
    }
    fn default() -> Self {
        FileHistory {}
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, FileHistory {})
    }
}

#[derive(Debug, Clone)]
struct HeaderBlock {}
impl Block for HeaderBlock {
    fn new() -> Self {
        HeaderBlock {}
    }
    fn default() -> Self {
        HeaderBlock {}
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, HeaderBlock {})
    }
}

#[derive(Debug, Clone)]
struct HeaderList {}
impl Block for HeaderList {
    fn new() -> Self {
        HeaderList {}
    }
    fn default() -> Self {
        HeaderList {}
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, HeaderList {})
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
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, ListData {})
    }
}

#[derive(Debug, Clone)]
struct SourceInformation {}
impl Block for SourceInformation {
    fn new() -> Self {
        SourceInformation {}
    }
    fn default() -> Self {
        SourceInformation {}
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, SourceInformation {})
    }
}

#[derive(Debug, Clone)]
struct TextBlock {}
impl Block for TextBlock {
    fn new() -> Self {
        TextBlock {}
    }
    fn default() -> Self {
        TextBlock {}
    }
    fn read(stream: &[u8]) -> (usize, Self) {
        (1, TextBlock {})
    }
}
