use crate::protocol::bencode as Bencoder;
use std::io::Error;
use torrex::bencode::Torrent;

pub trait TorrentParser {
    async fn from_bytes(bytes: &[u8]);
}

impl TorrentParser for Torrent {
    fn fr
}
