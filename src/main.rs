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

