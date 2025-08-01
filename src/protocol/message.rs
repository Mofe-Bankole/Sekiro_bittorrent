#[derive(Debug, Clone)]
pub enum PeerMessage {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(Vec<u8>),
    Request(u32, u32, u32), 
    Piece(u32, u32, Vec<u8>), 
    Cancel(u32, u32, u32), 
    Port(u16),
    KeepAlive,
    Handshake([u8; 68]),
}

