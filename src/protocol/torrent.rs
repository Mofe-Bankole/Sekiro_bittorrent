use crate::protocol::bencode as Bencoder;
use std::io::{Error , Result};
use torrex::bencode::Torrent;

pub trait TorrentParser {
    async fn from_bytes(bytes: &[u8]) where Self : Sized;
}

impl TorrentParser for Torrent {
    fn from_bytes(bytes : &[u8]) -> Result<()>{
        oK(())
    }
}
