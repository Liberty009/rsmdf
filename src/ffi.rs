use crate::mdf::{self, MDFFile, TimeChannel};
use crate::mdf3;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
// extern crate cbindgen;

#[repr(C)]
pub struct TimeSeries {
    time_values: *mut f64,
    time_length: u64,
    data_values: *mut f64,
    data_length: u64,
}

#[no_mangle]
pub extern "C" fn read_series(filepath: *const c_char, channel: *const c_char) -> TimeSeries {
    let channel_name = unsafe { CStr::from_ptr(channel) };
    let channel_name = channel_name.to_str().expect("msg");

    let file = unsafe { CStr::from_ptr(filepath) };

    let data_channel = mdf3::MDF3::new("filepath");
    let series = data_channel.read(0, 0, 1);

    let mut tv = series.timestamps;
    let mut dv = series.samples;

    TimeSeries {
        time_values: tv.as_mut_ptr(),
        time_length: tv.len() as u64,
        data_values: dv.as_mut_ptr(),
        data_length: dv.len() as u64,
    }
}

// #[no_mangle]
// pub extern "C" fn list_channels(filepath: *const c_char) {

// 	let mdf = MDF::new(unsafe{CStr::from_ptr(filepath)}.to_str().expect("Error"));
// }
