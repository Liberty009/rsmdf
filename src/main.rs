use mdf_rust::mdf3;
use std::{fs, str};


fn main() {
	
	let file = fs::read("example3.30.mdf").expect("msg");
	
	let (header, little_endian, position) = mdf3::read(&file);
	println!("File ID: {}", str::from_utf8(&header.file_id).expect("msg"));

	let (head, position) = mdf3::read_head(&file[position..], little_endian);
	println!("Block Type: {}", str::from_utf8(&head.block_type).expect(""));
	println!("Position: {}", position);
	println!("Project: {}", str::from_utf8(&head.project).expect(""));
	println!("Author: {}", str::from_utf8(&head.author).expect(""));

	println!("File Comment: {}", head.file_comment);

	//Try TXBLOCK with address in HDBLOCK
	let (test, _pos) = mdf3::TXBLOCK::read(&file[head.file_comment as usize..], little_endian);
	println!("Text: {}", str::from_utf8(&test.block_type).expect(""));
	println!("Text: {}", str::from_utf8(&test.text[0..]).expect(""));

	println!("PRBLOCK ADDRESS: {}", head.program_block);
	let (test_pr, _pos) = mdf3::PRBLOCK::read(&file[head.program_block as usize..], little_endian);
	println!("Program data: {}", str::from_utf8(&test_pr.program_data).expect(""));

}


#[cfg(test)]
mod tests {

use mdf_rust::mdf3;
	use mdf_rust::utils;
	#[test]
	fn idblock(){
		let id_data = [
			0x4D, 0x44, 0x46, 0x20, 
			0x20, 0x20, 0x20, 0x20, 
			0x33, 0x2E, 0x33, 0x30, 
			0x00, 0x00, 0x00, 0x00, 
			0x61, 0x6D, 0x64, 0x66, 
			0x36, 0x34, 0x34, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x4A, 0x01, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00,
		];

		let (id_block, position, endian) = mdf3::IDBLOCK::read(&id_data);

		assert_eq!(position, 64);
		assert_eq!(endian, true);
		assert!(utils::eq(&id_block.format_id, &[0x33, 0x2E, 0x33, 0x30, 
												0x00, 0x00, 0x00, 0x00,]));
		assert!(utils::eq(&id_block.program_id, &[0x61, 0x6D, 0x64, 0x66, 
												0x36, 0x34, 0x34, 0x00, ]));
		assert_eq!(id_block.default_float_format, 0);
		assert_eq!(id_block.version_number, 330);
		assert_eq!(id_block.code_page_number, 0);
		assert!(utils::eq(&id_block.reserved1, &[0, 0]));
		assert!(utils::eq(&id_block.reserved2, 
			&[
			00, 00, 00, 00, 
			00, 00, 00, 00, 
			00, 00, 00, 00, 
			00, 00, 00, 00, 
			00, 00, 00, 00, 
			00, 00, 00, 00, 
			00, 00, 00, 00, 
			00, 00,]))
		
	}

	#[test]
	fn hdblock(){
		let hd_data = [
			0x48, 0x44, 0xD0, 0x00, 
			0xD8, 0xDF, 0x10, 0x00, 
			0x10, 0x01, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x06, 0x00, 0x32, 0x32, 
			0x3A, 0x31, 0x31, 0x3A, 
			0x32, 0x30, 0x31, 0x38, 
			0x31, 0x34, 0x3A, 0x32, 
			0x36, 0x3A, 0x33, 0x35, 
			0x4A, 0x61, 0x63, 0x6B, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x29, 0x46, 0xF9, 
			0x75, 0x78, 0x69, 0x15, 
			0x00, 0x00, 0x00, 0x00, 
			0x4C, 0x6F, 0x63, 0x61, 
			0x6C, 0x20, 0x50, 0x43, 
			0x20, 0x52, 0x65, 0x66, 
			0x65, 0x72, 0x65, 0x6E, 
			0x63, 0x65, 0x20, 0x54, 
			0x69, 0x6D, 0x65, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x00, 0x00, 0x00, 0x00, 
			0x54, 0x58, 0xCC, 0x02, 
			0x3C, 0x48, 0x44, 0x63, 
			0x6F, 0x6D, 0x6D, 0x65, 
			0x6E, 0x74, 0x20, 0x78, 
			0x6D, 0x6C, 0x6E, 0x73, 
			0x3D, 0x22, 0x68, 0x74, 
			0x74, 0x70, 0x3A, 0x2F, 
			0x2F, 0x77, 0x77, 0x77, 
			0x2E, 0x61, 0x73, 0x61, 
			0x6D, 0x2E, 0x6E, 0x65, 
			0x74, 0x2F, 0x6D, 0x64, 
			0x66, 0x2F, 0x76, 0x34, 
			0x22, 0x3E, 0x3C, 0x54, 
			0x58, 0x3E, 0x44, 0x61, 
			0x74, 0x65, 0x3A, 0x20, 
			0x32, 0x32, 0x2E, 0x31, 
			0x31, 0x2E, 0x32, 0x30, 
			0x31, 0x38, 0x0D, 0x0A, 
			0x54, 0x69, 0x6D, 0x65, 
			0x3A, 0x20, 0x31, 0x35, 
			0x3A, 0x32, 0x37, 0x0D, 
			0x0A, 0x52, 0x65, 0x63, 
			0x6F, 0x72, 0x64, 0x69, 
			0x6E, 0x67, 0x20, 0x44, 
			0x75, 0x72, 0x61, 0x74, 
			0x69, 0x6F, 0x6E, 0x3A, 
			0x20, 0x30, 0x30, 0x3A, 
			0x30, 0x30, 0x3A, 0x31, 
			0x32, 0x0D, 0x0A, 0xA7, 
			0x40, 0x0D, 0x0A, 0x44, 
			0x61, 0x74, 0x61, 0x62, 
			0x61, 0x73, 0x65, 0x3A, 
			0x20, 0x54, 0x65, 0x73, 
			0x74, 0x0D, 0x0A, 0x45, 
			0x78, 0x70, 0x65, 0x72, 
			0x69, 0x6D, 0x65, 0x6E, 
			0x74, 0x3A, 0x20, 0x45, 
			0x78, 0x70, 0x65, 0x72, 
			0x69, 0x6D, 0x65, 0x6E, 
			0x74, 0x0D, 0x0A, 0x57, 
			0x6F, 0x72, 0x6B, 0x73, 
			0x70, 0x61, 0x63, 0x65, 
			0x3A, 0x20, 0x57, 0x6F, 
			0x72, 0x6B, 0x73, 0x70, 
			0x61, 0x63, 0x65, 0x0D, 
			0x0A, 0x44, 0x65, 0x76, 
			0x69, 0x63, 0x65, 0x73, 
			0x3A, 0x20, 0x45, 0x54, 
			0x4B, 0x20, 0x74, 0x65, 
			0x73, 0x74, 0x20, 0x64, 
			0x65, 0x76, 0x69, 0x63, 
			0x65, 0x3A, 0x31, 0x0D, 
			0x0A, 0x50, 0x72, 0x6F, 
			0x67, 0x72, 0x61, 0x6D, 
			0x20, 0x44, 0x65, 0x73, 
			0x63, 0x72, 0x69, 0x70, 
			0x74, 0x69, 0x6F, 0x6E, 
			0x3A, 0x20, 0x41, 0x53, 
			0x41, 0x50, 0x32, 0x5F, 
		];

		let (hd_block, position) = mdf3::HDBLOCK::read(&hd_data, true);

		println!("Length {}", position);
		assert_eq!(position, 208);

		assert_eq!(hd_block.block_size, 208);
		assert_eq!(hd_block.data_group_block, 1105880 );
		assert_eq!(hd_block.file_comment, 272);
		assert_eq!(hd_block.program_block, 0);
		assert_eq!(hd_block.data_group_number, 6);
		assert!(utils::eq(&hd_block.date, 
			&[
			0x32, 0x32, 0x3A, 0x31, 
			0x31, 0x3A, 0x32, 0x30, 
			0x31, 0x38,]));
		assert!(utils::eq(&hd_block.time, 
			&[
				0x31, 0x34, 0x3A, 0x32, 
				0x36, 0x3A, 0x33, 0x35,
			]));
		assert!(utils::eq(&hd_block.author, 
			&[
				0x4A, 0x61, 0x63, 0x6B, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00,
			]));
		assert!(utils::eq(
			&hd_block.department, 
			&[
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00,
			]
		));
		assert!(utils::eq(
			&hd_block.project, 
			&[
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00,
			]
		));
		assert!(utils::eq(
			&hd_block.subject, 
			&[
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00,
			]
		));
		assert_eq!(hd_block.timestamp, 1542896795439737088);
		assert_eq!(hd_block.utc_time_offset, 0 );
		assert_eq!(hd_block.time_quality, 0);
		assert!(utils::eq(
			&hd_block.timer_id, 
			&[
				0x4C, 0x6F, 0x63, 0x61, 
				0x6C, 0x20, 0x50, 0x43, 
				0x20, 0x52, 0x65, 0x66, 
				0x65, 0x72, 0x65, 0x6E, 
				0x63, 0x65, 0x20, 0x54, 
				0x69, 0x6D, 0x65, 0x00, 
				0x00, 0x00, 0x00, 0x00, 
				0x00, 0x00, 0x00, 0x00,
			]
		));
	}

	#[test]
	fn txblock(){

	}

	#[test]
	fn prblock(){

	}

	#[test]
	fn trblock(){

	}

	#[test]
	fn event(){

	}

	#[test]
	fn srblock(){

	}

	#[test]
	fn dgblock(){}

	#[test]
	fn cgblock(){}

	#[test]
	fn cnblock(){}

	#[test]
	fn ccblock(){}

	#[test]
	fn conversion_linear(){}

	#[test]
	fn conversion_poly(){}

	#[test]
	fn conversion_exponential(){}

	#[test]
	fn conversion_log(){}

	#[test]
	fn conversion_rational(){}

	#[test]
	fn conversion_tabular(){}

	#[test]
	fn table_entry(){}

	#[test]
	fn conversion_text_formula(){}

	#[test]
	fn conversion_text_table(){}

	#[test]
	fn text_table_entry(){}

	#[test]
	fn conversion_text_range_table(){}

	#[test]
	fn text_range(){}

	#[test]
	fn date_struct(){}

	#[test]
	fn time_struct(){}

	#[test]
	fn cdblock(){}

	#[test]
	fn signal(){}

	#[test]
	fn ceblock(){}

	#[test]
	fn supplement(){}

	#[test]
	fn dimblock(){}

	#[test]
	fn vector_block(){}

	
}

