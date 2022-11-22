#![allow(dead_code)]

use crate::chunk_type::ChunkType;
use crc::{Crc, CRC_32_ISO_HDLC};
use std::{
    fmt::{Display, Formatter},
    mem::size_of,
    str, u32,
};

const U_32_LEN: usize = size_of::<u32>();

#[derive(Default)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    chunk_data: Vec<u8>,
    crc: u32,
}


impl TryFrom<&[u8]> for Chunk {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut u32_dst = [0u8; 4];
        let mut start_index = 0;
        let mut end_index = U_32_LEN;
        // NOTE: Using non-inclusive ranges
        // TODO: this has to error somewhere
        // length field
        u32_dst.clone_from_slice(&value[start_index..end_index]);
        let length = u32::from_be_bytes(u32_dst);

        // chunk_type field
        start_index = end_index;
        end_index += U_32_LEN;
        u32_dst.clone_from_slice(&value[start_index..end_index]);
        let chunk_type = u32::from_le_bytes(u32_dst);

        // chunk_data field
        start_index = end_index; // skipping prev two fields
        let end_index = start_index + (length as usize);
        let mut chunk_data: Vec<u8> = vec![];
        chunk_data.extend(&value[start_index..end_index]);

        // crc field
        let start_index = end_index;
        let end_index = start_index + U_32_LEN;
        u32_dst.clone_from_slice(&value[start_index..end_index]);
        let crc = u32::from_be_bytes(u32_dst);

        let chunk_type = ChunkType::new(chunk_type);
        let chunk = Chunk::new(chunk_type, chunk_data);

        // check if length and crc(which includes chunk_type and chunk_data) are valid
        let is_valid_chunk = length == chunk.length() && crc == chunk.crc();

        match is_valid_chunk {
            true => Ok(chunk),
            false => Err("Given invalid byte array")
        }

    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            self.length(),
            self.chunk_type(),
            str::from_utf8(self.data()).unwrap(),
            self.crc()
        )
    }
}

impl Chunk {
    fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        let crc: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC); // spec mentions using iso-3309 crc method
        let type_bytes = chunk_type.bytes();
        let data_bytes = data.as_slice();
        let crc_bytes = [&type_bytes, data_bytes].concat();
        let checksum = crc.checksum(&crc_bytes);

        Chunk {
            length: data.len() as u32,
            chunk_type,
            chunk_data: data,
            crc: checksum,
        }
    }
    fn length(&self) -> u32 {
        self.length
    }
    fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }
    fn data(&self) -> &[u8] {
        &self.chunk_data
    }
    fn crc(&self) -> u32 {
        self.crc
    }
    fn data_as_string(&self) -> crate::MyResult<String> {
        match str::from_utf8(&self.chunk_data) {
            Ok(data_string) => Ok(String::from(data_string)),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn as_bytes(&self) -> Vec<u8> { 
        self.length()
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type().bytes().iter())
            .chain(self.chunk_data.iter())
            .chain(self.crc().to_be_bytes().iter())
            .copied()
            .collect()
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }

    #[test]
    pub fn test_chunk_as_bytes() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        let bytes = chunk.as_bytes();
        let chunk_from_bytes = Chunk::try_from(bytes.as_ref()).unwrap();
        assert!(chunk.length() == chunk_from_bytes.length());
        assert!(chunk.crc() == chunk_from_bytes.crc());
    }
}
