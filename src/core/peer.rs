use crate::protocol::message::PeerMessage;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Peer {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub address: SocketAddr,
    pub am_choking: bool,
    pub peer_choking: bool,
    pub am_interested: bool,
    pub peer_interested: bool,
    pub has_handshaked: bool,
    pub last_received: Option<PeerMessage>,
    pub last_sent: Option<PeerMessage>,
}
