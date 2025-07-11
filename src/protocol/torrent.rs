use std::io::Error;

use torrex::bencode::Torrent;

pub trait TorrentParser {
    async fn parse_to_torrent(path: &str) -> Result<Torrent, Error>;
}

impl TorrentParser for Torrent {
    async fn parse_to_torrent(path: &str) -> Result<Torrent, Error> {
        match Torrent::new(path) {
            Ok(torrent) => Ok(torrent),
            Err(e) => Err(Error::new(std::io::ErrorKind::Other, e.to_string())),
        }
    }
}
