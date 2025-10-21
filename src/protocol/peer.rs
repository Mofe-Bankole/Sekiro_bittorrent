use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Clone, Copy)]
pub struct Peer {
    pub ip: IpAddr,
    pub port: u16,
}

impl Peer {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }

    pub fn from(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}
