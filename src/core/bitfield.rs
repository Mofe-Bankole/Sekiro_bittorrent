pub struct Bitfield {
    pub pieces: bitfield::BitField,
    pub tx: flume::Sender<Vec<u8>>,
    pub rx: tokio::sync::mps::Receiver<Vec<u8>>,
}
