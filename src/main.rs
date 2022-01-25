use std::time::Instant;

use rsmdf::{mdf::MDFFile, mdf::MDF};

fn main() {
    let mdf = MDF::new("Larger_Test.mdf");

    mdf.list_channels();

    let start = Instant::now();
    // let test = mdf.read(0, 0, 1);

    let channel = mdf.search_channels("ASAM.M.SCALAR.SBYTE.IDENTICAL.DISCRETE");
    let channel = match channel {
        Ok(x) => x,
        Err(e) => panic!("{}", e),
    };
    let test = mdf.read_channel(channel);

    println!("Max Time: {}", test.max_time());
    println!("Took: {:?}", start.elapsed());
}
