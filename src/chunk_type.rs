#![allow(dead_code)]

use std::{mem, str, fmt::Display, str::FromStr};

const TYPE_LEN: usize = mem::size_of::<u32>();

#[derive(PartialEq, Eq, Debug)]
pub struct ChunkType {
    chunk_type: u32,
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = &'static str;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        if value.len() != TYPE_LEN {
            return Err("incorrect number of bytes in from_str parameter");
        }

        let mut result: u32 = 0;

        for (index, byte) in value.iter().enumerate() {
            let byte = byte.clone() as u32;
            result |= byte << (index * 8);
        }

		Ok(ChunkType::new(result))
    }
}

impl FromStr for ChunkType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != TYPE_LEN {
            return Err("incorrect number of bytes in from_str parameter");
        }

        let mut result: u32 = 0;

        for (index, byte) in s.as_bytes().iter().enumerate() {
			if !byte.is_ascii_alphabetic() {
				return Err("non-alphabetic character");
			}

			let byte = byte.clone() as u32;
            result |= byte << (index * 8);
        }

		Ok(ChunkType::new(result))
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.bytes();
        
		// write!(f, "{}", str::from_utf8(&self.bytes()).unwrap())
		write!(
            f,
            "{}{}{}{}",
            bytes[0] as char, bytes[1] as char, bytes[2] as char, bytes[3] as char
        )
    }
}

impl ChunkType {
    fn new(value: u32) -> ChunkType {
        ChunkType { chunk_type: value }
    }

    fn default() -> ChunkType {
        ChunkType { chunk_type: 0 }
    }

    fn bytes(&self) -> [u8; 4] {
        self.chunk_type.to_le_bytes()
    }

    fn is_valid(&self) -> bool {

		self.is_reserved_bit_valid() // must be zero to be valid per the current png standard 
	}
    // "A decoder encountering an unknown chunk in which the ancillary bit
    // is 1 can safely ignore the chunk and proceed to display the image. "
    // Probably good for hiding messages
    fn is_critical(&self) -> bool {
        (self.bytes()[0] >> 5) & 0x0001 == 0 // fifth bit of first byte encodes critical/ancillary
    }

    fn is_public(&self) -> bool {
        (self.bytes()[1] >> 5) & 0x0001 == 0 // fifth bit of second byte encodes public/private
    }

    fn is_reserved_bit_valid(&self) -> bool {
        (self.bytes()[2] >> 5) & 0x0001 == 0 // fifth bit of third  byte reserved for future use
    }

    fn is_safe_to_copy(&self) -> bool {
        (self.bytes()[3] >> 5) & 0x0001 == 1 // fifth bit of fourth  byte reserved for safe/unsafe to copy for editors
    }
}

// pub struct ChunkType {
// 	length: u32,
// 	chunk_type: [u8; 4],
// 	chunk_data: Vec<u8>,
// 	crc: u32
// }

/*
    Tests
*/
#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
		let chunk = ChunkType::from_str("RuSt").unwrap();
		assert!(chunk.is_valid());
    }
    
    #[test]
    pub fn test_invalid_chunk_is_valid() {
		let chunk = ChunkType::from_str("Rust").unwrap();
		assert!(!chunk.is_valid());
		
		let chunk = ChunkType::from_str("Ru1t");
		assert!(chunk.is_err());
    }
    
    #[test]
    pub fn test_chunk_type_string() {
		let chunk = ChunkType::from_str("RuSt").unwrap();
		assert_eq!(&chunk.to_string(), "RuSt");
    }
    
    #[test]
    pub fn test_chunk_type_trait_impls() {
		let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
		let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
		let _chunk_string = format!("{}", chunk_type_1);
		let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
