pub enum PeerMessage {
    Connected,
    Intrested,
    Unintrested,
    HandShake,
    Bitfield,
    Request,
    Piece,
    Cancel,
    Port,
    KeepAlive,
    Unknown,
}

impl PeerMessage {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        match bytes[0] {
            0 => Ok(PeerMessage::Connected),
            1 => Ok(PeerMessage::Intrested),
            2 => Ok(PeerMessage::Unintrested),
            3 => Ok(PeerMessage::HandShake),
            4 => Ok(PeerMessage::Bitfield),
            5 => Ok(PeerMessage::Request),
            6 => Ok(PeerMessage::Piece),
            7 => Ok(PeerMessage::Cancel),
            8 => Ok(PeerMessage::Port),
            9 => Ok(PeerMessage::KeepAlive),
            _ => Err(Error::UnknownMessage),
        }
    }
}
