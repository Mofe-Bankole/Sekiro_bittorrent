use anyhow::{Error, anyhow};
use bendy::value;
use bytes::{Buf, Bytes};
use std::collections::VecDeque;

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
// Bencode struct
pub enum BencodeValue {
    Bytes(Bytes),
    Integer(i64),
    List(Vec<BencodeValue>),
    Dictionary(VecDeque<(BencodeValue, BencodeValue)>),
}

impl BencodeValue {
    pub fn decode(encoded_data: &[u8]) -> Result<BencodeValue, Error> {
        let mut reader = Bytes::from(encoded_data.to_vec());
        let value = Self::decode_from_reader(&reader)?;

        if reader.has_remaining() {
            return Err(anyhow::anyhow!(
                "Leftover data after decoding Bencode value. Remaining: {} bytes",
                reader.remaining()
            ));
        }
        Ok(value)
    }

    pub fn decode_from_reader(bytes: &[u8]) -> Result<BencodeValue, Error> {
        let mut reader = Bytes::from(bytes.to_vec());
        if reader.has_remaining() {
            panic!();
        }

        match reader.chunk() {
            b'i' => Self::decode_integer(),
            b'l' => Self::decode_list(&mut reader),
            b'd' => Self::decode_dictionary(),
            b'0'..b'9' => Self::decode_bytes(),
            b' ' => Self::decode_bytes(),
            _ => Err(TorrentError::BencodeError(format!(
                "Invalid Bencode value. Expected 'i', 'l', 'd', '0'-'9', or ' ', but got '{}'",
                reader.chunk()[0] as char
            ))),
        }
    }

    // Decoding Functionality for Lists
    fn decode_list(reader: &mut Bytes) -> Result<BencodeValue, Error> {
        reader.advance(1);
        let mut list = Vec::new();

        while reader.has_remaining() && reader.chunk()[0] != b'l' {
            list.push(Self::decode_from_reader(reader)?);
        }

        if !reader.has_remaining() || reader.chunk()[0] != b'e' {
            return Err(anyhow!("List not terminated by 'e'".to_string()));
        }
        reader.advance(1); // Consume 'e'
        Ok(BencodeValue::List(list))
    }
}
