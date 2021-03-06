use std::mem;

use super::block::Block;
use super::block_header::*;
use crate::utils;

use super::block::LinkedBlock;
use super::mdf4_file::link_extract;
use super::{
    mdf4_enums::{ChannelType, DataType, SyncType},
    tx_block::Txblock,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Cnblock {
    header: BlockHeader,
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
    reserved1: u8,
    attachment_nr: u16,
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

impl LinkedBlock for Cnblock {
    fn next(&self, stream: &[u8], little_endian: bool) -> Option<Self> {
        if self.cn_cn_next == 0 {
            None
        } else {
            let (_, block) = Cnblock::read(stream, self.cn_cn_next as usize, little_endian);
            Some(block)
        }
    }

    fn list(&self, stream: &[u8], little_endian: bool) -> Vec<Self> {
        let mut all = Vec::new();

        let next = self.next(stream, little_endian);

        all.push(self.clone());
        match next {
            None => {}
            Some(block) => all.append(&mut block.list(stream, little_endian)),
        }

        all
    }
}

impl Cnblock {
    pub fn byte_offset(&self) -> usize {
        self.bit_offset as usize
    }

    pub fn data_type(&self) -> DataType {
        self.data_type
    }

    pub fn data_type_len(&self) -> usize {
        self.data_type.len()
    }

    pub fn comment(self, stream: &[u8], little_endian: bool) -> String {
        let mut name = "".to_string();

        if matches!(self.channel_type, ChannelType::Master) {
            name = "time".to_string();
        } else if self.cn_tx_name != 0 {
            let (_pos, tx) = Txblock::read(stream, self.cn_tx_name as usize, little_endian);

            name = tx.text();
        }

        name
    }

    pub fn channel_type(&self) -> ChannelType {
        self.channel_type.clone()
    }
}

impl Block for Cnblock {
    fn new() -> Self {
        Cnblock {
            header: BlockHeader::create("##CN", 50, 0),
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
            reserved1: 0,
            attachment_nr: 0,
            min_raw_value: 0.0,
            max_raw_value: 0.0,
            lower_limit: 0.0,
            upper_limit: 0.0,
            lower_ext_limit: 0.0,
            upper_ext_limit: 0.0,
        }
    }
    fn default() -> Self {
        Cnblock {
            header: BlockHeader::create("##CN", 50, 0),
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
            reserved1: 0,
            attachment_nr: 0,
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
        let reserved1 = utils::read(stream, little_endian, &mut pos);
        let attachment_nr = utils::read(stream, little_endian, &mut pos);
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

        let default_x = flags == 1 << 12;

        let mut cn_default_x = Vec::with_capacity(3);
        if default_x {
            for _i in 0..3 {
                cn_default_x.push(addresses.remove(0));
            }
        }

        (
            pos,
            Cnblock {
                header,
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
                reserved1,
                attachment_nr,
                min_raw_value,
                max_raw_value,
                lower_limit,
                upper_limit,
                lower_ext_limit,
                upper_ext_limit,
            },
        )
    }

    fn byte_len(&self) -> usize {
        let mut length = self.header.byte_len()
            + mem::size_of_val(&self.cn_cn_next)
            + mem::size_of_val(&self.cn_composition)
            + mem::size_of_val(&self.cn_tx_name)
            + mem::size_of_val(&self.cn_si_source)
            + mem::size_of_val(&self.cn_cc_conversion)
            + mem::size_of_val(&self.cn_data)
            + mem::size_of_val(&self.cn_md_unit)
            + mem::size_of_val(&self.cn_md_comment)
            + mem::size_of_val(&self.channel_type)
            + mem::size_of_val(&self.sync_type)
            + mem::size_of_val(&self.data_type)
            + mem::size_of_val(&self.bit_offset)
            + mem::size_of_val(&self.byte_offset)
            + mem::size_of_val(&self.bit_count)
            + mem::size_of_val(&self.flags)
            + mem::size_of_val(&self.pos_invalidation_bit)
            + mem::size_of_val(&self.precision)
            + mem::size_of_val(&self.reserved1)
            + mem::size_of_val(&self.attachment_nr)
            + mem::size_of_val(&self.min_raw_value)
            + mem::size_of_val(&self.max_raw_value)
            + mem::size_of_val(&self.lower_limit)
            + mem::size_of_val(&self.upper_limit)
            + mem::size_of_val(&self.lower_ext_limit)
            + mem::size_of_val(&self.upper_ext_limit);

        if !&self.cn_at_reference.is_empty() {
            length += mem::size_of_val(&self.cn_at_reference[0]) * self.cn_at_reference.len();
        }
        if !&self.cn_default_x.is_empty() {
            length += mem::size_of_val(&self.cn_default_x[0]) * self.cn_default_x.len();
        }

        length
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use crate::mdf4::{block::Block, cn_block::Cnblock};

    static RAW: [u8; 160] = [
        0x23, 0x23, 0x43, 0x4E, 0x00, 0x00, 0x00, 0x00, 0xA0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x28, 0x63, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x45, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x18, 0x62, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x5B, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x46, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0xB0, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x01,
        0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn read() {
        let (pos, cn) = Cnblock::read(&RAW, 0, true);

        assert_eq!(pos, 160);
        assert_eq!(cn.cn_cn_next, 25384);
        assert_eq!(cn.cn_composition, 0);
        assert_eq!(cn.cn_tx_name, 17728);
        assert_eq!(cn.cn_si_source, 25112);
        assert_eq!(cn.cn_cc_conversion, 23488);
        assert_eq!(cn.cn_md_unit, 17984);
        assert_eq!(cn.cn_md_comment, 17840);
    }

    #[test]
    fn byte_len() {
        let (pos, cn) = Cnblock::read(&RAW, 0, true);

        assert_eq!(pos, 160);
        assert_eq!(24, cn.header.byte_len());
        assert_eq!(8, mem::size_of_val(&cn.cn_cn_next));
        assert_eq!(8, mem::size_of_val(&cn.cn_composition));
        assert_eq!(8, mem::size_of_val(&cn.cn_tx_name));
        assert_eq!(8, mem::size_of_val(&cn.cn_si_source));
        assert_eq!(8, mem::size_of_val(&cn.cn_cc_conversion));
        assert_eq!(8, mem::size_of_val(&cn.cn_data));
        assert_eq!(8, mem::size_of_val(&cn.cn_md_unit));
        assert_eq!(8, mem::size_of_val(&cn.cn_md_comment));
        assert_eq!(0, cn.cn_at_reference.len());
        assert_eq!(0, cn.cn_default_x.len());
        assert_eq!(1, mem::size_of_val(&cn.channel_type));
        assert_eq!(1, mem::size_of_val(&cn.sync_type));
        assert_eq!(1, mem::size_of_val(&cn.data_type));
        assert_eq!(1, mem::size_of_val(&cn.bit_offset));
        assert_eq!(4, mem::size_of_val(&cn.byte_offset));
        assert_eq!(4, mem::size_of_val(&cn.bit_count));
        assert_eq!(4, mem::size_of_val(&cn.flags));
        assert_eq!(4, mem::size_of_val(&cn.pos_invalidation_bit));
        assert_eq!(1, mem::size_of_val(&cn.precision));
        assert_eq!(8, mem::size_of_val(&cn.min_raw_value));
        assert_eq!(8, mem::size_of_val(&cn.max_raw_value));
        assert_eq!(8, mem::size_of_val(&cn.lower_limit));
        assert_eq!(8, mem::size_of_val(&cn.upper_limit));
        assert_eq!(8, mem::size_of_val(&cn.lower_ext_limit));
        assert_eq!(8, mem::size_of_val(&cn.upper_ext_limit));
        assert_eq!(160, cn.byte_len());
    }
}
