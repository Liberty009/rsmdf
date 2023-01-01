use crate::utils;

use super::block::Block;
use super::block_header::*;
use super::mdf4_enums::{EventCause, EventSyncType, EventType, RangeType};
use super::mdf4_file::link_extract;

#[derive(Debug, Clone, PartialEq)]
pub struct EVBlock {
    header: BlockHeader,

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
    ev_reserved: [u8; 3],

    ev_scope_count: u32,

    ev_attachment_count: u16,

    ev_creator_index: u16,

    ev_sync_base_value: i64,

    ev_sync_factor: f64,
}

impl Block for EVBlock {
    fn new() -> Self {
        Self {
            header: BlockHeader::create("##EV", 50, 0),
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
            ev_reserved: [0_u8; 3],
            ev_scope_count: 0_u32,
            ev_attachment_count: 0_u16,
            ev_creator_index: 0_u16,
            ev_sync_base_value: 0_i64,
            ev_sync_factor: 0_f64,
        }
    }
    fn default() -> Self {
        Self {
            header: BlockHeader::create("##EV", 50, 0),
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
            ev_reserved: [0_u8; 3],
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

        let ev_reserved: [u8; 3] = utils::read(stream, little_endian, &mut pos);

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
                header,
                ev_ev_next,
                ev_ev_parent,
                ev_ev_range,
                ev_tx_name,
                ev_md_comment,
                ev_scope,
                ev_at_reference,
                ev_type,
                ev_reserved,
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

    fn byte_len(&self) -> usize {
        todo!()
    }
}
