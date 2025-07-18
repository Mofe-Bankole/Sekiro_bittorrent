use std::io::Error;
use torrex::bencode::Torrent;

pub trait TorrentParser {
    async fn from_bytes(bytes: &[u8]);
}

impl TorrentParser for Torrent {
    async fn from_bytes(bytes: &[u8]) {
        let info_bytes = crate::protocol::bencode::decode(bytes);
    }
}
