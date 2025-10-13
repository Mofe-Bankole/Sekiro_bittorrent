use crate::protocol::{bencode::BencodeValue, peer::Peer};
use anyhow::anyhow;
use color_eyre::eyre::Ok;
use std::result::Result::*;
use std::time::{SystemTime, UNIX_EPOCH};
#[derive(Debug, Clone)]
/// Data representation of a Tracker Request
pub struct TrackerRequest {
    pub info_hash: [u8; 20],
    pub left: u64,
    /// bytes uploaded
    pub uploaded: u64,
    /// bytes downloaded
    pub downloaded: u64,
    pub port: u16,
    pub compact: bool,
    pub event: Option<TrackerEvent>,
}

#[derive(Debug, Clone)]
pub enum TrackerEvent {
    Started,
    Completed,
    Stopped,
}

impl TrackerEvent {
    fn as_str(&self) -> &str {
        match self {
            TrackerEvent::Started => "started",
            TrackerEvent::Completed => "completed",
            TrackerEvent::Stopped => "stopped",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrackerResponse {
    pub interval: u64,
    /// Peers
    pub peers: Vec<Peer>,
    pub complete: Option<u64>,   // Number of seeders
    pub incomplete: Option<u64>, // Number of leechers
    pub tracker_id: Option<String>,
}

/// Data representation of a tracker
///
/// The tracker is a central server that helps peers find each other.
///
/// It doesn't host files - it just maintains a list of who's downloading/uploading what.
pub struct Tracker {
    announce_url: String,
    /// Peer id
    peer_id: [u8; 20],
}

impl Tracker {
    pub fn new(announce_url: String) -> Self {
        // Generate a random peer ID
        let peer_id = Self::generate_peer_id();

        Self {
            announce_url,
            peer_id,
        }
    }

    /// Generates a random id for a peer
    pub fn generate_peer_id() -> [u8; 20] {
        let mut peer_id = [0u8; 20];

        peer_id[0..8].copy_from_slice(b"-RS0000-");

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for (i, byte) in peer_id[8..].iter_mut().enumerate() {
            *byte = ((timestamp >> (i * 8)) & 0xFF) as u8;
        }

        peer_id
    }

    pub async fn announce(&self, request: TrackerRequest) -> Result<TrackerResponse> {
        let url = self.build_announce_url(request);

        println!("Contacting tracker at: {}", self.announce_url);
        let response = reqwest::get(url).await?;

        // Check HTTP status
        if !response.status().is_success() {
            return Err(anyhow!("Tracker returned error: {}", response.status()));
        }

        // Read body bytes
        let body = response
            .bytes()
            .await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

        println!("Response bytes: {:?}", body);
        todo!("Parse response into TrackerResponse")
    }

    fn build_announce_url(&self, req: TrackerRequest) -> Result<String, anyhow::Error> {
        let mut url = self.announce_url.clone();

        url.push('?');

        url.push_str("?info_hash=");
        url.push_str(&Self::url_encode(&req.info_hash));

        url.push_str("&peer_id=");
        url.push_str(&Self::url_encode(&self.peer_id));

        url.push_str(&format!("&port={}", req.port));

        url.push_str(&format!("&uploaded={}", req.uploaded));
        url.push_str(&format!("&downloaded={}", req.downloaded));

        url.push_str(&format!("&left={}", req.left));
        url.push_str(&format!("&compact={}", if req.compact { 1 } else { 0 }));
        // url.push_str(&format!("&event={}");

        if let Some(event) = req.event {
            url.push_str(&format!("&event={}", event.as_str()));
        }

        Ok(url)
    }

    /// URL encode bytes for tracker request
    fn url_encode(bytes: &[u8]) -> String {
        let mut result = String::new();

        for &byte in bytes {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                _ => {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }

        result
    }
    fn parse_tracker_response(&self, data: &[u8]) -> Result<TrackerResponse, anyhow::Error> {
        let mut value = BencodeValue::decode(data);

        let mut interval = None;
        let mut peers_data = None;
        let mut complete = None;
        let mut incomplete = None;
        let mut tracker_id = None;
        let mut failure_reason = None;

        let dict = match value {
            Ok(BencodeValue::Dictionary(pairs)) => pairs,
            _ => return Err(anyhow!("Tracker response is not a dictionary")),
        };

        let mut i = 0;
        while i + 1 < dict.len() {
            if let BencodeValue::Bytes(key_bytes) = &dict[i] {
                let key_str = String::from_utf8_lossy(key_bytes);

                match key_str.as_ref() {
                    "interval" => {
                        if let BencodeValue::Integer(val) = dict[i + 1] {
                            interval = Some(val as u64);
                        }
                    }
                    "peers" => {
                        peers_data = Some(&dict[i + 1]);
                    }
                    "complete" => {
                        if let BencodeValue::Integer(val) = dict[i + 1] {
                            complete = Some(val as u64);
                        }
                    }
                    "incomplete" => {
                        if let BencodeValue::Integer(val) = dict[i + 1] {
                            incomplete = Some(val as u64);
                        }
                    }
                    "tracker id" => {
                        if let BencodeValue::Bytes(val) = &dict[i + 1] {
                            tracker_id = Some(String::from_utf8_lossy(val).to_string());
                        }
                    }
                    "failure reason" => {
                        if let BencodeValue::Bytes(val) = &dict[i + 1] {
                            failure_reason = Some(String::from_utf8_lossy(val).to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(reason) = failure_reason {
            return Err(anyhow!("Tracker error: {}", reason));
        }
    }
    pub fn get_peer_id(&self) -> [u8; 20] {
        self.peer_id
    }
}
