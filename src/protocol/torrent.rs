use crate::protocol::bencode as Bencoder;
use std::io::{ Result };
use std::path::PathBuf;
use bytes::Bytes;

pub trait TorrentParser {
    fn from_bytes(bytes: &[u8]) -> Result<Self> where Self: Sized;
    fn from_file(path: &PathBuf) -> Result<Self> where Self: Sized;
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
    fn from_bytes(bytes: &[u8]) -> Result<()> {
        let mut reader = Bytes::from(bytes.to_vec());
        let mut bencoded_value = Bencoder::BencodeValue::decode(bytes)?;
    }

    fn from_file(path: &PathBuf) -> Result<Self> {
      let bytes = std::fs::read(path)?;
      Self::from_bytes(&bytes)
    }
}

