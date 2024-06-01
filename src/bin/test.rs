use std::{fs, io::{stdout, Write}};
use smol_file::SmolBlob;

fn main() {
    let input: String = "hej då".to_owned();
    //println!("> input text len: {}", input.len());
    print!("e: {:?}", input);
    fs::write("raw.bin", &input).unwrap();
    
    let encoded: SmolBlob = SmolBlob::encode(&input);
    //println!("> encoded blob len: {}", encoded.len());
    fs::write("smol.bin", &encoded.buffer()).unwrap();

    let headerless = encoded.buffer_headerless();
    fs::write("smol-headerless.bin", &headerless).unwrap();

    let decoded: String = SmolBlob::decode_headerless(&headerless).unwrap();
    //println!("< decoded text len: {}", decoded.len());
    println!(" - d: {:?}", decoded);
    fs::write("unsmol.bin", &decoded).unwrap();

    stdout().flush().unwrap();

    // they should be equal, otherwise something is wrong
    assert_eq!(input, decoded);
    println!("Success!");
}