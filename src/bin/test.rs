use std::fs;
use smol_file::SmolBlob;

fn main() {
    println!("testing with input \"test\"");
    println!();

    let input: String = "test".to_owned();
    println!("> input text len: {}", input.len());
    println!("> input text: {}", input);
    fs::write("raw.bin", &input).unwrap();
    
    let encoded: SmolBlob = SmolBlob::encode(&input);
    println!("> encoded blob len: {}", encoded.len());
    fs::write("smol.bin", &encoded.buffer()).unwrap();

    let decoded: String = SmolBlob::decode_blob(&encoded).unwrap();
    println!("< decoded text len: {}", decoded.len());
    println!("< decoded text: {:?}", decoded);
    fs::write("unsmol.bin", &decoded).unwrap();

    // they should be equal, otherwise something is wrong
    assert_eq!(input, decoded);
}