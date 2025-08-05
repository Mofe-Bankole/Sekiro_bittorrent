use crate::protocol::bencode::{self as Bencoder, BencodeValue};
use anyhow::{Result, anyhow};
use bytes::Bytes;

pub trait TorrentParser {
    fn extract_announce(bytes: &[u8]) -> Result<String>;
    fn extract_info_hash(bytes: &[u8]) -> Result<[u8; 20]>;
    fn encode_bencode(value : &BencodeValue , but &mut Vec<u8>) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct Torrent {
    pub announce: String,
    pub info_hash: [u8; 20],
    pub piece_length: usize,
    pub pieces: Vec<[u8; 20]>,
    pub name: String,
    pub length: usize,
    pub files: Option<Vec<TorrentFile>>,
}

#[derive(Debug, Clone)]
pub struct TorrentFile {
    pub path: Vec<String>,
    pub length: usize,
}

impl TorrentParser for Torrent {
    fn extract_announce(bytes: &[u8]) -> Result<String> {
        let value = Bencoder::BencodeValue::decode(bytes).unwrap();
        let dict = match value {
            BencodeValue::Dictionary(pairs) => pairs,
            _ => return Err(anyhow!("Torrent is not a dictionary at the top level")),
        };

        let mut i = 0;
        while i + 1 < dict.len() {
            if let BencodeValue::Bytes(key_bytes) = &dict[i] {
                if key_bytes.as_ref() == b"announce" {
                    if let BencodeValue::Bytes(val_bytes) = &dict[i + 1] {
                        let announce = String::from_utf8(val_bytes.to_vec())
                            .map_err(|_| anyhow!("Invalid UTF-8 in announce string"))?;
                        return Ok(announce);
                    } else {
                        return Err(anyhow!("'announce' is not a byte string"));
                    }
                }
            }
            i += 2;
        }

        Err(anyhow!("Announce field not found in dictionary"));
    }

    fn extract_info_hash(bytes: &[u8]) -> Result<[u8; 20]> {
        let mut reader = Bytes::from(bytes.to_vec());
        let value = BencodeValue::decode_from_reader(&mut reader);
        let dict = match value {
            BencodeValue::Dictionary(pairs) => pairs,
            _ => return Err(anyhow!("Torrent is not a dictionary at the top level")),
        };

        let mut i = 0;
        while i + 1 < dict.len() {
            if let BencodeValue::Bytes(info_bytes) = &dict[i]{
                if info_bytes.as_ref() == b"info"{
                    let info = &dict[i + 1];

                }
            }
        }

        Err(anyhow!("Info field not found in dictionary"))
    }
}
