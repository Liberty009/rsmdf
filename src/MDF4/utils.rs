/// # Safety
///
/// This function should not be called before the horsemen are ready.
unsafe fn str_from_u8_nul_utf8_unchecked(utf8_src: &[u8]) -> &str {
    let nul_range_end = utf8_src
        .iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len()); // default to length if no `\0` present
    ::std::str::from_utf8_unchecked(&utf8_src[0..nul_range_end])
}

pub fn str_from_u8(c_string: &[u8]) -> String{
	unsafe {
		str_from_u8_nul_utf8_unchecked(c_string).to_string()
	}
}

