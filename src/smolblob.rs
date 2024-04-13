use std::io::{Cursor, Read};

use crate::utils;

pub struct SmolBlob {
    version: u64,
    data: Vec<u8>
}

impl SmolBlob {
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
    
        return SmolBlob { version: 1, data };
    }
    
    pub fn decode(input: &mut Vec<u8>) -> Result<String, String> {
        if input.len() < 4 {
            return Err("missing smol data, it is truncated?".into());
        }
        let mut curs = Cursor::new(input);
        
        let mut magic = [0u8; 4];
        curs.read_exact(&mut magic).unwrap();
    
        if &magic != b"smol" {
            return Err("not an smol buffer".into());
        }
    
        let header_size = leb128::read::unsigned(&mut curs).unwrap();
        let mut header = vec![0u8; header_size as usize];
        curs.read_exact(&mut header).unwrap();
    
        let data_size = leb128::read::unsigned(&mut curs).unwrap();
        let mut data = vec![0u8; data_size as usize];
        curs.read_exact(&mut data).unwrap();
    
        let mut out_string = String::new();
        let mut current_bit: usize = 0;
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
}