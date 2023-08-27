use std::fs;

const ALPHABET: &str = " abcdefghijklmnopqrstuvwxyz.!?12";

fn main() {
    let input = "hello. this is a test".to_owned();

    //let input = fs::read_to_string("bible2022.txt").unwrap();
    println!("{}", input.len());

    fs::write("raw.bin", &input).unwrap();

    let encoded = encode(&input);
    println!("encoded len: {}", encoded.len());

    fs::write("smol.bin", &encoded).unwrap();

    let decoded = decode(encoded);
    println!("decoded len: {}", decoded.len());
    //fs::write("out.bin", result).unwrap();

    fs::write("unsmol.bin", decoded).unwrap();
}

fn encode(str: &String) -> Vec<u8> {
    let mut buffer = vec![];

    let mut current: u16 = 0;
    let mut offset: usize = 0;
    for char in str.to_lowercase().chars().into_iter() {
        let index = char_to_index(char);
        current |= (index as u16) << offset;
        offset += 5;
        if offset >= 8 {
            buffer.push((current & 0xff) as u8);
            current = (current & 0xff00) >> 8;
            offset -= 8;
        }
    }
    //println!("{}", offset);
    if offset > 0 {
        buffer.push(current as u8);
    }

    return buffer;
}

fn decode(input: Vec<u8>) -> String {
    if input.len() == 0 {
        panic!("decode length is zero");
    }

    
    let mut out_string = String::new();
    let mut offset: usize = 0;
    loop {
        let current_byte_index = offset / 8;
        if current_byte_index >= input.len() {
            break;
        }
        let first_byte: u8 = input[current_byte_index];
        let secound_byte: u8 = if current_byte_index + 1 < input.len() {
            input[current_byte_index + 1]
        } else {
            0
        };
        let current: u16 = ((secound_byte as u16) << 8) | first_byte as u16;
        
        //println!("{:016b}", current);

        let char_index = (current & (31 << (offset%8))) >> (offset%8);
        let char = index_to_char(char_index as usize);
        //println!("{:?}, at offset {}", char, offset);
        out_string.push(char);
        offset += 5;
    }

    out_string
}

fn char_to_index(search_char: char) -> usize {
    for (index, char) in ALPHABET.chars().enumerate() {
        if search_char == char {
            return index;
        }
    }
    return 0;
}

fn index_to_char(search_index: usize) -> char {
    for (index, char) in ALPHABET.chars().enumerate() {
        if search_index == index {
            return char;
        }
    }
    return ' ';
}
