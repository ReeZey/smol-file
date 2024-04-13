mod utils;
mod smolblob;

use std::fs;
use smolblob::SmolBlob;

fn main() {
    let input = "test".to_owned();
    //println!("INPUT LENGTH: {}", input.len());
    fs::write("raw.bin", &input).unwrap();

    let encoded = SmolBlob::encode(&input);
    fs::write("smol.bin", &encoded.buffer()).unwrap();

    let decoded = SmolBlob::decode(&mut encoded.buffer()).unwrap();
    //println!("decoded len: {}", decoded.len());
    //println!("decoded text: {:?}", decoded);
    fs::write("unsmol.bin", decoded).unwrap();
}