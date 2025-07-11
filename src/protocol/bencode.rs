use bendy::{Bencode, BencodeError, Torrent};

async fn decode_bencode() -> Result<Torrent, BencodeError> {
    let decoder = Decoder;:new();

}
