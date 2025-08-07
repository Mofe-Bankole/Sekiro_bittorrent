use crate::protocol::bencode::{self as Bencoder, BencodeValue};
use anyhow::{Result, anyhow};
use bytes::Bytes;
use sha1::{Digest, Sha1};

pub trait TorrentParser {
    fn extract_announce(bytes: &[u8]) -> Result<String>;
    fn extract_info_hash(bytes: &[u8]) -> Result<[u8; 20]>;
    fn encode_bencode(value: &BencodeValue, buf: &mut Vec<u8>) -> Result<()>;
    fn extract_name(bytes: &[u8]) -> Result<String>;
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
    fn extract_announce(bytes: &[u8]) -> Result<String, anyhow::Error> {
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

        Err(anyhow!("Announce field not found in dictionary"))
    }
    fn extract_name(bytes: &[u8]) -> Result<String> {
        let mut reader = Bytes::from(bytes.to_vec());
        let value = BencodeValue::decode_from_reader(&mut reader);
        let dict = match value{
            OK(BencodeValue::Dictionary(pairs)) => pairs,
            _ => return Err(anyhow!("Torrent is not a dictionary at the top level"))
        }
    }
    fn extract_info_hash(bytes: &[u8]) -> Result<[u8; 20]> {
        let mut reader = Bytes::from(bytes.to_vec());
        let value = BencodeValue::decode_from_reader(&mut reader);
        let dict = match value {
            Ok(BencodeValue::Dictionary(pairs)) => pairs,
            _ => return Err(anyhow!("Torrent is not a dictionary at the top level")),
        };

        let mut i = 0;
        while i + 1 < dict.len() {
            if let BencodeValue::Bytes(info_bytes) = &dict[i] {
                if info_bytes.as_ref() == b"info" {
                    let info = &dict[i + 1];
                    let mut buf = Vec::new();

                    Self::encode_bencode(info, &mut buf);
                    let hash = Sha1::digest(&buf);

                    let mut hash_bytes = [0u8; 20];
                    hash_bytes.copy_from_slice(&hash);
                    Ok(hash_bytes);
                }
            }
            i += 2;
        }

        Err(anyhow!("Info field not found in dictionary"))
    }

    // Helper Functions
    fn encode_bencode(value: &BencodeValue, buf: &mut Vec<u8>) -> Result<()> {
        match value {
            BencodeValue::Integer(i) => {
                buf.extend_from_slice(b"i");
                buf.extend_from_slice(i.to_string().as_bytes());
                buf.extend_from_slice(b"e");
            }
            BencodeValue::List(ls) => {
                buf.extend_from_slice(b"l");
                for item in ls {
                    Self::encode_bencode(item, buf)?;
                }
                buf.extend_from_slice(b"e");
            }
            BencodeValue::Dictionary(dict) => {
                buf.extend_from_slice(b"d");
                let mut i = 0;
                while i < dict.len() {
                    let key = &dict[i];
                    let val = &dict[i + 1];
                    Self::encode_bencode(key, buf)?;
                    Self::encode_bencode(val, buf)?;
                    i += 2;
                }
                buf.extend_from_slice(b"e");
            }
            BencodeValue::Bytes(bytes) => {
                buf.extend_from_slice(bytes.len().to_string().as_bytes());
                buf.extend_from_slice(b":");
                buf.extend_from_slice(bytes);
            }
        }
        Ok(())
    }
}
