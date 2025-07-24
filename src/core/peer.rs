use crate::protocol::message::PeerMessage;

#[derive(Debug, Clone)]
pub struct Peer {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub address: std::net::SocketAddr,
    pub am_choking: bool,
    pub am_intrested: bool,
    pub peer_intrested: bool,
    pub message: PeerMessage,
}

impl Peer {
    pub async fn new(id: Option<u64>, name: Option<String>, address: std::net::SocketAddr) -> Self {
        Self {
            id,
            name,
            address,
            am_choking: true,
            am_intrested: false,
            peer_intrested: false,
        }
    }
}
