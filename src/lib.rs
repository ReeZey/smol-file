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
//! the last two (1 and 2) have special functions which is:  [CURRENTLY NOT IMPLEMENTED]  
//! (1): enters number mode - all characters read will be interpreted as their number part. [A = 1, B = 2 ... I = 9]  
//! (2): makes the next procceding character uppercase or special action in some cases. example: newlines are formatted `"2 "`
//!   
//! see also [`SmolBlob`]

use std::io::{ Cursor, Read };
use anyhow::{ Result, Error };
mod utils;

// CHANGE THIS EACH RELEASE
const VERSION: u64 = 1;

pub struct SmolBlob {
    version: u64,
    data: Vec<u8>
}

/// [`SmolBlob`] is a chunk of smol data
impl SmolBlob {
    /// converts the [`SmolBlob`] data into a valid file buffer
    pub fn buffer(&self) -> Vec<u8> {
        let mut buffer = vec![];
        buffer.extend(b"smol");

        let mut header = vec![];
        leb128::write::unsigned(&mut header, self.version).unwrap();

        leb128::write::unsigned(&mut buffer, header.len() as u64).unwrap();
        buffer.extend(header);
        leb128::write::unsigned(&mut buffer, self.data.len() as u64).unwrap();
        buffer.extend(self.data.clone());

        buffer
    }

    /// returns the inner buffer length **ONLY**, not to be confused by [`SmolBlob::buffer`].len()
    pub fn len(&self) -> usize {
        self.buffer().len()
    }

    /// encodes a [`String`] and returns a [`SmolBlob`]
    /// # example
    ///
    /// ```
    /// let encoded: SmolBlob = SmolBlob::encode(&input);
    /// fs::write("smol.bin", &encoded.buffer()).unwrap();
    /// ```
    pub fn encode(str: &String) -> SmolBlob {
        let mut data = vec![];
    
        let mut current: u16 = 0;
        let mut offset: usize = 0;
        for char in str.to_lowercase().chars().into_iter() {
            let index = utils::char_to_index(char);
            current |= (index as u16) << offset;
            offset += 5;
            if offset >= 8 {
                data.push((current & 0xff) as u8);
                current = (current & 0xff00) >> 8;
                offset -= 8;
            }
        }
    
        if offset > 0 {
            data.push(current as u8);
        }
    
        return SmolBlob { version: VERSION, data };
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
    
        let header_size = leb128::read::unsigned(&mut curs)?;
        let mut header = vec![0u8; header_size as usize];
        curs.read_exact(&mut header)?;
    
        let data_size = leb128::read::unsigned(&mut curs)?;
        let mut data = vec![0u8; data_size as usize];
        curs.read_exact(&mut data)?;
    
        let mut out_string = String::new();
        let mut current_bit: u8 = 0;
        let mut byte_index: usize = 0;
        loop {
            let mut current: u8 = (data[byte_index] >> current_bit) & 0x1F;
    
            if current_bit >= 4 {
                if byte_index == (data.len() - 1) {
                    break;
                } 
    
                current |= (data[byte_index + 1]  << (8 - current_bit)) & 0x1F;
            }
    
            out_string.push(utils::index_to_char(current as usize));
    
            current_bit += 5;
            if current_bit >= 8 {
                byte_index += 1;
                current_bit -= 8;
            }
        }
    
        Ok(out_string)
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
}