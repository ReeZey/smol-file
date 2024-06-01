//! # smol - 5 bit encoding file format
//! what is smol? smol is a file format that compresses text into 5 bits per letter instead of the normal 8 bits.
//! this is achived by having a charset lesser than 32 chars so all letters fits nicely into 5 bits (32 values). 
//! this makes all sort of problem such as an byte being 8 so multiple letters will overlap eachother
//! but this is all handeled through this library for ease of use.  
//!   
//! this is not made for any production applications only as an hobby.  
//!   
//! ## how does this work?  
//! this is the entire charset  
//! ```
//! " abcdefghijklmnopqrstuvwxyz.!?12"  
//!  ^ space
//! ```  
//!  
//! first 30 chars are normal (space, alphabet, ! and ?)  
//!   
//! the last two (1 and 2) have special functions which is:  
//! (1): enters number mode - all characters read will be interpreted as their number part. [A = 1, B = 2 ... I = 9]  
//! (2): makes the next procceding character uppercase or special action in some cases. example: newlines are formatted `"2 "`
//!   
//! see also [`SmolBlob`]

use std::io::{ Cursor, Read };
use anyhow::{ Error, Ok, Result };
use utils::{char_to_index, index_to_char};
mod utils;

// CHANGE THIS EACH RELEASE
const VERSION: u64 = 3;

#[allow(dead_code)]
pub struct SmolBlob {
    version: u64,
    buffer: Vec<u8>,

    size: u64,

    current: u16,
    offset: u8,

    number_mode: bool,
    super_mode: bool,
}

impl Default for SmolBlob {
    fn default() -> Self {
        Self { 
            version: VERSION, 
            buffer: vec![], 
            size: 0, 
            current: 0, 
            offset: 0, 
            number_mode: false, 
            super_mode: false,
        }
    }
}

/// [`SmolBlob`] is a chunk of smol data
impl SmolBlob {
    /// converts the [`SmolBlob`] data into a valid file buffer
    pub fn buffer(&self) -> Vec<u8> {
        let mut buffer = vec![];
        buffer.extend(b"smol");

        let mut header = vec![];
        leb128::write::unsigned(&mut header, self.version).unwrap();
        leb128::write::unsigned(&mut header, self.size).unwrap();

        leb128::write::unsigned(&mut buffer, header.len() as u64).unwrap();
        buffer.extend(header);
        leb128::write::unsigned(&mut buffer, self.buffer.len() as u64).unwrap();
        buffer.extend(self.buffer.clone());

        buffer
    }

    /// return nececcary stuff for reconstruction  
    /// useful for IRC/generic datastream transfer, where header is not needed
    pub fn buffer_headerless(&self) -> Vec<u8> {
        let mut buffer = vec![];
        leb128::write::unsigned(&mut buffer, self.size).unwrap();
        buffer.extend(self.buffer.clone());

        buffer
    }

    /// encodes a [`String`] and returns a [`SmolBlob`]
    /// # example
    ///
    /// ```
    /// let encoded: SmolBlob = SmolBlob::encode(&input);
    /// fs::write("smol.bin", &encoded.buffer()).unwrap();
    /// ```
    pub fn encode(str: &String) -> SmolBlob {
        let mut blob = SmolBlob::default();
        for char in str.chars().into_iter() {
            if char.is_numeric() {
                if !blob.number_mode {
                    blob.push_char('1');
                    blob.number_mode = true;
                }
                let number: u32 = char.to_digit(10).unwrap();
                blob.push_char(index_to_char(number as usize));
            } else {
                if blob.number_mode {
                    blob.push_char('1');
                    blob.number_mode = false;
                }

                //fuck windows
                if char == '\r' {
                    continue;
                }

                if char == '\n' {
                    blob.push_char('2');
                    blob.push_char(' ');
                    continue;
                }

                let invalid_char = char_to_index(char);
                if invalid_char.is_none() {
                    let mut buffer = vec![0; char.len_utf8()];
                    char.encode_utf8(&mut buffer);

                    blob.push_char('2');
                    blob.push_char('1');

                    
                    let length = char.len_utf8();

                    for char in buffer {
                        blob.buffer.push(char);
                    }
                    continue;
                }
                
                if char.is_uppercase() {
                    blob.push_char('2');
                }
                
                blob.push_char(char.to_ascii_lowercase());
            } 
        }

        if blob.number_mode {
            blob.push_char('1');
            blob.number_mode = false;
        }
    
        if blob.offset > 0 {
            blob.buffer.push(blob.current as u8);
        }
    
        return blob;
    }

    fn push_char(&mut self, char: char) {
        self.size += 1;

        let char= utils::char_to_index(char);

        let index = char.unwrap_or(0);
        self.current |= (index as u16) << self.offset;
        self.offset += 5;
        if self.offset >= 8 {
            self.buffer.push((self.current & 0xff) as u8);
            self.current = (self.current & 0xff00) >> 8;
            self.offset -= 8;
        }
    }

    /// decodes a [`SmolBlob::buffer`] and returns a [`String`]  
    /// # example
    ///
    /// ```
    /// let decoded: String = SmolBlob::decode(&encoded.buffer()).unwrap();
    /// fs::write("unsmol.bin", &decoded).unwrap();
    /// ```
    pub fn decode(input: &Vec<u8>) -> Result<String, Error> {
        if input.len() < 4 {
            return Err(Error::msg("file is too small, is it truncated?"));
        }
        let mut curs = Cursor::new(input);
        
        let mut magic = [0u8; 4];
        curs.read_exact(&mut magic)?;
    
        if &magic != b"smol" {
            return Err(Error::msg("invalid file magic, this is not an smol file"));
        }
    
        let _header_size = leb128::read::unsigned(&mut curs)?;
        let version = leb128::read::unsigned(&mut curs)?;

        let buffer_size;
        let size = match version {
            1 => {
                buffer_size = leb128::read::unsigned(&mut curs)?;
                buffer_size
            }
            _ => {
                let size = leb128::read::unsigned(&mut curs)?;
                buffer_size = leb128::read::unsigned(&mut curs)?;
                size
            }
        };

        let mut blob = SmolBlob::default();
        blob.buffer = vec![0u8; buffer_size as usize];
        curs.read_exact(&mut blob.buffer)?;
    
        let mut out_string = String::new();

        for _ in 0..size {
            let char = blob.read_char();
            
            if char.is_numeric() && !blob.super_mode {
                match char {
                    '1' => {
                        blob.number_mode = !blob.number_mode;
                    }
                    '2' => {
                        blob.super_mode = true;
                    }
                    _ => {
                        return Err(Error::msg("invalid data when decoding"));
                    }
                }
                continue;
            }
            
            if blob.number_mode {
                let num: u32 = char_to_index(char).unwrap_or(0) as u32;
                out_string.push(char::from_digit(num, 10).unwrap());
                continue;
            }

            if blob.super_mode {
                match char {
                    ' ' => {
                        out_string.push('\n');
                    }
                    '1' => {
                        let size: u32 = blob.buffer.remove(0) as u32 | 
                                    (blob.buffer.remove(0) as u32) << 8 | 
                                    (blob.buffer.remove(0) as u32) << 16 | 
                                    (blob.buffer.remove(0) as u32) << 24;

                        let mut buffer = vec![];
                        for _ in 0..size {
                            buffer.push(blob.buffer.remove(0))
                        }
                        out_string += &String::from_utf8(buffer).unwrap();
                    }
                    _ => {
                        out_string.push(char.to_ascii_uppercase());
                    }
                }
                blob.super_mode = false;
                continue;
            }

            out_string.push(char);
        }
    
        Ok(out_string)
    }

    fn read_char(&mut self) -> char {
        let current_byte = self.buffer.get(0).unwrap();

        let mut current: u8 = (current_byte >> self.offset) & 0x1F;
    
        if self.offset >= 4 {
            current |= (self.buffer.get(1).unwrap()  << (8 - self.offset)) & 0x1F;
        }

        self.offset += 5;
        if self.offset >= 8 {
            self.buffer.remove(0);
            self.offset -= 8;
        }

        let char = utils::index_to_char(current as usize);

        return char;
    }

    /// decodes a [`SmolBlob`] and returns a [`String`]  
    /// internally this is the same as [`SmolBlob::decode`] but with [`SmolBlob`] instead of [`Vec<u8>`] buffer
    /// # example
    ///
    /// ```
    /// let decoded: String = SmolBlob::decode_blob(&encoded).unwrap();
    /// fs::write("unsmol.bin", &decoded).unwrap();
    /// ```
    pub fn decode_blob(input: &SmolBlob) -> Result<String, Error> {
        return SmolBlob::decode(&input.buffer());
    }

    /// decodes a [`SmolBlob::buffer_headerless`] and returns a [`String`]  
    /// # example
    ///
    /// ```
    /// let decoded: String = SmolBlob::decode_headerless(&encoded.buffer_headerless()).unwrap();
    /// fs::write("unsmol.bin", &decoded).unwrap();
    /// ```
    pub fn decode_headerless(input: &Vec<u8>) -> Result<String, Error> {
        let mut curs = Cursor::new(input);
        let mut blob = SmolBlob::default();
        blob.size = leb128::read::unsigned(&mut curs)?;
        curs.read_to_end(&mut blob.buffer)?;
        
        return SmolBlob::decode_blob(&blob);
    }
}