use crate::protocol::{bencode::BencodeValue, peer::Peer};
use anyhow::{Result, anyhow};
use color_eyre::{eyre::Ok, owo_colors::OwoColorize};

use std::time::{SystemTime, UNIX_EPOCH};

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
pub struct TrackerRequest {
    pub info_hash: [u8; 20],
    pub left: u64,
    pub uploaded: u64,
    pub downloaded: u64,
    pub port: u16,
    pub compact: bool,
    pub event: Option<TrackerEvent>,
}

#[derive(Debug, Default, Clone)]
pub struct TrackerResponse {
    pub interval: u64,
    pub peers: Vec<Peer>,
    pub complete: Option<u64>,   // No of complete pieces
    pub incomplete: Option<u64>, // No of incomplete pieces
    pub tracker_id: Option<String>,
}

#[derive(Debug)]
pub struct Tracker {
    announce_url: String,
    peer_id: [u8; 20],
}

impl Tracker {
    pub fn new(announce_url: String) -> Self {
        let peer_id = Self::generate_peer_id();
        Self {
            announce_url,
            peer_id,
        }
    }

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

    pub async fn announce(
        &self,
        request: TrackerRequest,
    ) -> Result<TrackerResponse, anyhow::Error> {
        let url = self.build_announce_url(&request);
        println!("Contacting tracker at : {}", self.announce_url.green()).bright_black();

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Tracker returned error: {}", response.status()));
        }

        let body = response
            .bytes()
            .await
            .map_err(|e| anyhow!("Failed to read response body : {}", e))?;

        println!("Response bytes: {:?}", body);

        self.parse_tracker_response(&body)
    }

    fn build_announce_url(&self, req: &TrackerRequest) -> String {
        let mut url = format!(
            "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",
            self.announce_url,
            Self::url_encode(&req.info_hash),
            Self::url_encode(&self.peer_id),
            req.port,
            req.uploaded,
            req.downloaded,
            req.left,
            if req.compact { 1 } else { 0 }
        );

        if let Some(event) = &req.event {
            url.push_str(&format!("&event={}", event.as_str()));
        }

        url
    }

    fn url_encode(bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|&b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    (b as char).to_string()
                }
                _ => format!("%{:02X}", b),
            })
            .collect()
    }

    pub fn parse_tracker_response(&self, data: &[u8]) -> Result<TrackerResponse> {
        let value = BencodeValue::decode(data)?;

        let dict = match value {
            BencodeValue::Dictionary(map) => map,
            _ => return Err(anyhow!("Tracker response is not a dictionary")),
        };

        let mut interval = None;
        let mut peers_data = None;
        let mut complete = None;
        let mut incomplete = None;
        let mut tracker_id = None;
        let mut failure_reason = None;

        // Iterate over dictionary entries
        for (key_bytes, val) in dict {
            let key = String::from_utf8_lossy(&key_bytes).to_string();

            match key.as_str() {
                "interval" => {
                    if let BencodeValue::Integer(v) = val {
                        interval = Some(v as u64);
                    }
                }
                "peers" => peers_data = Some(val),
                "complete" => {
                    if let BencodeValue::Integer(v) = val {
                        complete = Some(v as u64);
                    }
                }
                "incomplete" => {
                    if let BencodeValue::Integer(v) = val {
                        incomplete = Some(v as u64);
                    }
                }
                "tracker id" => {
                    if let BencodeValue::Bytes(bytes) = val {
                        tracker_id = Some(String::from_utf8_lossy(&bytes).to_string());
                    }
                }
                "failure reason" => {
                    if let BencodeValue::Bytes(bytes) = val {
                        failure_reason = Some(String::from_utf8_lossy(&bytes).to_string());
                    }
                }
                _ => {}
            }
        }

        if let Some(reason) = failure_reason {
            return Err(anyhow!("Tracker failure: {}", reason));
        }

        Ok(TrackerResponse {
            interval: interval.unwrap_or(0),
            peers: vec![], // left as placeholder (don’t add parsing logic)
            complete,
            incomplete,
            tracker_id,
        })
    }

    pub fn parse_peers(peers_value: &BencodeValue) -> Result<Vec<Peer>> {
        match peers_value {
            // Dictionary list form (non-compact)
            BencodeValue::List(list) => {
                let mut peers = Vec::new();
                for item in list {
                    if let BencodeValue::Dictionary(map) = item {
                        let ip = match map.get(0) {
                            Some(BencodeValue::Bytes(ip)) => ip.clone(),
                            _ => continue,
                        };
                        let port = match map.get(1) {
                            Some(BencodeValue::Integer(port)) => port.clone(),
                            _ => continue,
                        };

                        peers.push(Peer::new(ip, port as u16));
                    }
                }
                Ok(peers)
            }
            // TODO : Implement binary format
            _ => Err(anyhow!("Invalid Peer Format")),
        }
    }

    pub fn get_peer_id(&self) -> [u8; 20] {
        self.peer_id
    }
}
