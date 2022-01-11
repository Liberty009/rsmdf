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

    let data = mdf::MDF::new(file.to_str().expect("msg"));
	let chan = data.search_channels(channel_name);
    let channel = match chan {
		        Ok(x) => x,
		        Err(e) => panic!("{}", e),
		    };
	let mut test = data.read_channel(channel);

    TimeSeries {
        time_values: test.timestamps.as_mut_ptr(),
        time_length: test.timestamps.len() as u64,
        data_values: test.samples.as_mut_ptr(),
        data_length: test.samples.len() as u64,
    }
}

// #[no_mangle]
// pub extern "C" fn list_channels(filepath: *const c_char) {

// 	let mdf = MDF::new(unsafe{CStr::from_ptr(filepath)}.to_str().expect("Error"));
// }
