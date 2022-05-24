use std::time::Instant;

use rsmdf::{mdf::MDFFile, mdf::MDF};

fn main() {
    let mdf = MDF::new("./example_files/ASAP2_Demo_V171.mf4");

    mdf.list_channels();

    let start = Instant::now();
    // let test = mdf.read(0, 0, 1);

    // let channel = mdf.search_channels("ASAM.M.SCALAR.SBYTE.IDENTICAL.DISCRETE");
    // let channel = match channel {
    //     Ok(x) => x,
    //     Err(e) => panic!("{}", e),
    // };
    // let test = mdf.read_channel(channel);

    for channel in &mdf.channels() {
        let test = &mdf.read_channel(channel);
        println!("{}", test.comment);
    }

    let test = mdf.read_channel(&mdf.channels[0]);

    println!("Max Time: {}", test.max_time());
    println!("Took: {:?}", start.elapsed());
}
