use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct Peer {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub address: std::net::SocketAddr,
    pub am_choking: bool,
    pub am_intrested: bool,
    pub peer_intrested: bool,
    pub pieces: bitfield::BitField,
    pub tx: flume::Sender<Vec<u8>>,
    pub rx: flume::Receiver<Vec<u8>>,
}

pub enum PeerState {
    Connected,
    Disconnected,
}

pub enum PeerMessga {
    Intrested,
    Unintrested,
    HandShake,
    KeepAlive,
    UnableToAccept,
}

#[tokio::main]
impl Peer {
    pub async fn new(id: Option<u64>, name: Option<String>, address: std::net::SocketAddr) -> Self {
        let (tx, rx) = flume::unbounded(5000);
        Self {
            id,
            name,
            address,
            am_choking: true,
            am_intrested: false,
            peer_intrested: false,
            pieces: bitfield::BitField::new(0),
            tx,
            rx,
        }
    }
}
