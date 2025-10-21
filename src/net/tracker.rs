use crate::protocol::{bencode::BencodeValue, peer::Peer};
use anyhow::{Result, anyhow};
use bytes::Bytes;
use color_eyre::{eyre::Ok, owo_colors::OwoColorize};
use serde::Serialize;
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

#[derive(Debug, Clone)]
pub struct TrackerResponse {
    pub interval: u64,
    pub peers: Vec<Peer>,
    pub complete: Option<u64>,   // No of complete pieces
    pub incomplete: Option<u64>, // No of incomplete pieces
    pub tracker_id: Option<String>,
}

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

    pub async fn announce(&self, request: TrackerRequest) -> Result<TrackerResponse> {
        let url = self.build_announce_url(&request);
        println!(
            "Contacting tracker at: {}",
            self.announce_url.bright_black()
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Tracker returned error: {}", response.status()));
        }

        let body = response
            .bytes()
            .await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

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

    fn parse_tracker_response(&self, data: &[u8]) -> Result<TrackerResponse> {
        let value = BencodeValue::decode(data)?;
        let interval = None;
        let peers = None;
        let complete = None;
        let compact = None;
        let incomplete = None;
        let dict = match value {
            BencodeValue::Dictionary(map) => map,
            _ => return Err(anyhow!("Tracker response is not a dictionary")),
        };

        for i in 0..=dict.len() {
            if let BencodeValue::Bytes(key_bytes) = &dict[i] {
                if key_bytes == "interval" {
                    if let BencodeValue::Integer(i) = &dict[i + 1] {
                        Some(i)
                        return Err(anyhow!("Missing interval in tracker response"));
                    }
                } else if key_bytes == "compact" {
                    if let BencodeValue::Integer(i) = &dict[i + 1] {
                        return Err(anyhow!("Missing compact in tracker response"));
                    }
                } else if key_bytes == "incomplete" {
                    if let BencodeValue::Integer(i) = &dict[i + 1] {
                        peers = Some(())
                        return Err(anyhow!("Missing incomplete in tracker response"));
                    }
                } else if key_bytes == "peers" {
                    if let BencodeValue::Integer(i) = &dict[i + 1] {
                        return Err(anyhow!("Missing Peers in tracker response"));
                    }
                }
            }
        }

        let interval = match &dict[0] {
            BencodeValue::Integer(i) => *i as u64,
            _ => return Err(anyhow!("Missing interval in tracker response")),
        };

        let complete = match dict.get("complete") {
            Some(BencodeValue::Integer(i)) => Some(*i as u64),
            _ => None,
        };

        let incomplete = match dict.get("incomplete") {
            Some(BencodeValue::Integer(i)) => Some(*i as u64),
            _ => None,
        };

        let tracker_id = match dict.get("tracker id") {
            Some(BencodeValue::Bytes(v)) => Some(String::from_utf8_lossy(v).to_string()),
            _ => None,
        };

        let peers_value = dict
            .get("peers")
            .ok_or_else(|| anyhow!("Missing peers in tracker response"))?;

        let peers = Self::parse_peers(peers_value)?;

        Ok(TrackerResponse {
            interval,
            peers,
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
                        let ip = match map.get("ip") {
                            Some(BencodeValue::Bytes(ip)) => ip.clone(),
                            _ => continue,
                        };
                        let port = match map.get("port") {
                            Some(BencodeValue::Integer(p)) => *p as u16,
                            _ => continue,
                        };
                        peers.push(Peer::new(ip, port));
                    }
                }
                Ok(peers)
            }
            // Compact binary form
            BencodeValue::Bytes(bytes) => {
                let mut peers = Vec::new();
                for chunk in bytes.chunks_exact(6) {
                    let ip = chunk[0..4].to_vec();
                    let port = u16::from_be_bytes([chunk[4], chunk[5]]);
                    peers.push(Peer::new(ip, port));
                }
                Ok(peers)
            }
            _ => Err(anyhow!("Invalid peers format")),
        }
    }

    pub fn get_peer_id(&self) -> [u8; 20] {
        self.peer_id
    }
}
