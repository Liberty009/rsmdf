use super::block::Block;
use super::block_header::*;
use super::mdf4::link_extract;
use crate::utils;

#[derive(Debug, Clone)]
struct Cablock {
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
impl Block for Cablock {
    fn new() -> Self {
        Cablock {
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
        Cablock {
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
            Cablock {
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

    fn byte_len(&self) -> usize {
        todo!()
    }
}
