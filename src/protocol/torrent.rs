use std::usize;

use crate::protocol::bencode::{self as Bencoder, BencodeValue};
use anyhow::{Result, anyhow};
use bytes::Bytes;
use sha1::{Digest, Sha1};

pub trait TorrentParser {
    fn extract_announce(bytes: &[u8]) -> Result<String>;
    fn extract_info_hash(bytes: &[u8]) -> Result<[u8; 20]>;
    fn encode_bencode(value: &BencodeValue, buf: &mut Vec<u8>) -> Result<()>;
    fn extract_name(bytes: &[u8]) -> Result<String>;
    fn extract_piece_length(bytes: &[u8]) -> Result<usize>;
    fn extract_pieces(bytes: &[u8]) -> Result<Vec<[u8; 20]>>;
    fn extract_length(bytes: &[u8]) -> Result<usize>;
    fn extract_files(bytes: &[u8]) -> Result<Option<Vec<TorrentFile>>>;
}

#[derive(Debug, Clone)]
pub struct Torrent {
    pub announce: String,
    pub info_hash: [u8; 20],
    /// Lenght of a single piece in the torrent ( 256 - 1024kb  , might be 2,3mb depending on creator)
    pub piece_length: usize,
    /// Pieces of the torrent
    pub pieces: Vec<[u8; 20]>,
    pub name: String,
    pub length: usize,
    pub files: Option<Vec<TorrentFile>>,
}

#[derive(Debug, Clone)]
pub struct TorrentFile {
    pub path: Vec<String>,
    pub length: usize,
}

impl Torrent {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let announce = Self::extract_announce(bytes)?;
        let info_hash = Self::extract_info_hash(bytes)?;
        let name = Self::extract_name(bytes)?;
        let piece_length = Self::extract_piece_length(bytes)?;
        let pieces = Self::extract_pieces(bytes)?;
        let length = Self::extract_length(bytes)?;
        let files = Self::extract_files(bytes)?;

        Ok(Torrent {
            announce,
            info_hash,
            piece_length,
            pieces,
            name,
            length,
            files,
        })
    }
}

impl TorrentParser for Torrent {
    fn extract_announce(bytes: &[u8]) -> Result<String, anyhow::Error> {
        let value = Bencoder::BencodeValue::decode(bytes)?;
        let dict = match value {
            BencodeValue::Dictionary(pairs) => pairs,
            _ => return Err(anyhow!("Torrent is not a dictionary at the top level")),
        };

        let mut i = 0;
        while i + 1 < dict.len() {
            if let BencodeValue::Bytes(key_bytes) = &dict[i] {
                if key_bytes.as_ref() == b"announce" {
                    if let BencodeValue::Bytes(val_bytes) = &dict[i + 1] {
                        let announce = String::from_utf8(val_bytes.to_vec())
                            .map_err(|_| anyhow!("Invalid UTF-8 in announce string"))?;
                        return Ok(announce);
                    } else {
                        return Err(anyhow!("'announce' is not a byte string"));
                    }
                }
            }
            i += 2;
        }

        Err(anyhow!("Announce field not found in dictionary"))
    }

    fn extract_name(bytes: &[u8]) -> Result<String> {
        let mut reader = Bytes::from(bytes.to_vec());
        let value = BencodeValue::decode_from_reader(&mut reader);
        let dict = match value {
            Ok(BencodeValue::Dictionary(pairs)) => pairs,
            _ => return Err(anyhow!("Torrent is not a dictionary at the top level")),
        };

        let mut i = 0;
        while i + 1 < dict.len() {
            if let BencodeValue::Bytes(key_bytes) = &dict[i] {
                if key_bytes.as_ref() == b"info" {
                    if let BencodeValue::Dictionary(info_dict) = &dict[i + 1] {
                        let mut j = 0;
                        while j + 1 < info_dict.len() {
                            if let BencodeValue::Bytes(name_key_bytes) = &info_dict[j] {
                                if name_key_bytes.as_ref() == b"name" {
                                    if let BencodeValue::Bytes(name_bytes) = &info_dict[j + 1] {
                                        let name = String::from_utf8(name_bytes.to_vec())
                                            .map_err(|_| anyhow!("Invalid UTF-8 in name string"))?;
                                        return Ok(name);
                                    } else {
                                        return Err(anyhow!("'name' is not a byte string"));
                                    }
                                }
                            }
                            j += 2;
                        }
                    }
                }
            }
            i += 2;
        }

        Err(anyhow!("Name field not found in info dictionary"))
    }

    fn extract_info_hash(bytes: &[u8]) -> Result<[u8; 20]> {
        let mut reader = Bytes::from(bytes.to_vec());
        let value = BencodeValue::decode_from_reader(&mut reader);
        let dict = match value {
            Ok(BencodeValue::Dictionary(pairs)) => pairs,
            _ => return Err(anyhow!("Torrent is not a dictionary at the top level")),
        };

        let mut i = 0;
        while i + 1 < dict.len() {
            if let BencodeValue::Bytes(info_bytes) = &dict[i] {
                if info_bytes.as_ref() == b"info" {
                    let info = &dict[i + 1];
                    let mut buf = Vec::new();

                    Self::encode_bencode(info, &mut buf)?;
                    let hash = Sha1::digest(&buf);

                    let mut hash_bytes = [0u8; 20];
                    hash_bytes.copy_from_slice(&hash);
                    return Ok(hash_bytes);
                }
            }
            i += 2;
        }

        Err(anyhow!("Info field not found in dictionary"))
    }

    fn extract_piece_length(bytes: &[u8]) -> Result<usize> {
        let mut reader = Bytes::from(bytes.to_vec());
        let value = BencodeValue::decode_from_reader(&mut reader);
        let dict = match value {
            Ok(BencodeValue::Dictionary(pairs)) => pairs,
            _ => return Err(anyhow!("Torrent is not a dictionary at the top level")),
        };

        let mut i = 0;
        while i + 1 < dict.len() {
            if let BencodeValue::Bytes(key_bytes) = &dict[i] {
                if key_bytes.as_ref() == b"info" {
                    if let BencodeValue::Dictionary(info_dict) = &dict[i + 1] {
                        let mut j = 0;
                        while j + 1 < info_dict.len() {
                            if let BencodeValue::Bytes(piece_key_bytes) = &info_dict[j] {
                                if piece_key_bytes.as_ref() == b"piece length" {
                                    if let BencodeValue::Integer(piece_len) = info_dict[j + 1] {
                                        return Ok(piece_len as usize);
                                    } else {
                                        return Err(anyhow!("piece length is not an integer"));
                                    }
                                }
                            }
                            j += 2;
                        }
                    }
                }
            }
            i += 2;
        }

        Err(anyhow!("Piece Length Was NOT FOUND 404"))
    }

    fn extract_pieces(bytes: &[u8]) -> Result<Vec<[u8; 20]>> {
        let mut reader = Bytes::from(bytes.to_vec());
        let value = BencodeValue::decode_from_reader(&mut reader);
        let dict = match value {
            Ok(BencodeValue::Dictionary(pairs)) => pairs,
            _ => return Err(anyhow!("Torrent is not a dictionary at the top level")),
        };

        let mut i = 0;
        while i + 1 < dict.len() {
            if let BencodeValue::Bytes(key_bytes) = &dict[i] {
                if key_bytes.as_ref() == b"info" {
                    if let BencodeValue::Dictionary(info_dict) = &dict[i + 1] {
                        let mut j = 0;
                        while j + 1 < info_dict.len() {
                            if let BencodeValue::Bytes(pieces_key_bytes) = &info_dict[j] {
                                if pieces_key_bytes.as_ref() == b"pieces" {
                                    if let BencodeValue::Bytes(pieces_bytes) = &info_dict[j + 1] {
                                        let pieces_data = pieces_bytes.as_ref();
                                        if pieces_data.len() % 20 != 0 {
                                            return Err(anyhow!(
                                                "Pieces data length is not a multiple of 20"
                                            ));
                                        }

                                        let mut pieces = Vec::new();
                                        for chunk in pieces_data.chunks(20) {
                                            let mut piece_hash = [0u8; 20];
                                            piece_hash.copy_from_slice(chunk);
                                            pieces.push(piece_hash);
                                        }
                                        return Ok(pieces);
                                    } else {
                                        return Err(anyhow!("'pieces' is not a byte string"));
                                    }
                                }
                            }
                            j += 2;
                        }
                    }
                }
            }
            i += 2;
        }

        Err(anyhow!("Pieces field not found in info dictionary"))
    }

    fn extract_length(bytes: &[u8]) -> Result<usize> {
        let mut reader = Bytes::from(bytes.to_vec());
        let value = BencodeValue::decode_from_reader(&mut reader);
        let dict = match value {
            Ok(BencodeValue::Dictionary(pairs)) => pairs,
            _ => return Err(anyhow!("Torrent is not a dictionary at the top level")),
        };

        let mut i = 0;
        while i + 1 < dict.len() {
            if let BencodeValue::Bytes(key_bytes) = &dict[i] {
                if key_bytes.as_ref() == b"info" {
                    if let BencodeValue::Dictionary(info_dict) = &dict[i + 1] {
                        let mut j = 0;
                        while j + 1 < info_dict.len() {
                            if let BencodeValue::Bytes(length_key_bytes) = &info_dict[j] {
                                match length_key_bytes.as_ref() {
                                    b"length" => {
                                        if let BencodeValue::Integer(length) = info_dict[j + 1] {
                                            return Ok(length as usize);
                                        } else {
                                            return Err(anyhow!("Length is not a usize"));
                                        }
                                    }
                                    b"files" => {
                                        if let BencodeValue::List(files_list) = &info_dict[j + 1] {
                                            let mut total_length = 0;
                                            for file in files_list {
                                                if let BencodeValue::Dictionary(file_dict) = file {
                                                    let mut k = 0;
                                                    while k + 1 < file_dict.len() {
                                                        if let BencodeValue::Bytes(
                                                            file_length_key,
                                                        ) = &file_dict[k]
                                                        {
                                                            if file_length_key.as_ref() == b"length"
                                                            {
                                                                if let BencodeValue::Integer(
                                                                    file_length,
                                                                ) = file_dict[k + 1]
                                                                {
                                                                    total_length +=
                                                                        file_length as usize;
                                                                }
                                                            }
                                                        }
                                                        k += 2;
                                                    }
                                                }
                                            }
                                            return Ok(total_length);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            j += 2;
                        }
                    }
                }
            }
            i += 2;
        }
        Err(anyhow!("Length field not found in info dictionary"))
    }

    fn extract_files(bytes: &[u8]) -> Result<Option<Vec<TorrentFile>>> {
        let mut reader = Bytes::from(bytes.to_vec());
        let value = BencodeValue::decode_from_reader(&mut reader);
        let dict = match value {
            Ok(BencodeValue::Dictionary(pairs)) => pairs,
            _ => return Err(anyhow!("Torrent is not a dictionary at the top level")),
        };

        let mut i = 0;
        while i + 1 < dict.len() {
            if let BencodeValue::Bytes(key_bytes) = &dict[i] {
                if key_bytes.as_ref() == b"info" {
                    if let BencodeValue::Dictionary(info_dict) = &dict[i + 1] {
                        let mut j = 0;
                        while j + 1 < info_dict.len() {
                            if let BencodeValue::Bytes(files_key_bytes) = &info_dict[j] {
                                if files_key_bytes.as_ref() == b"files" {
                                    if let BencodeValue::List(files_list) = &info_dict[j + 1] {
                                        let mut torrent_files = Vec::new();
                                        for file_value in files_list {
                                            if let BencodeValue::Dictionary(file_dict) = file_value
                                            {
                                                let mut file_length = 0;
                                                let mut file_path = Vec::new();

                                                let mut k = 0;
                                                while k + 1 < file_dict.len() {
                                                    if let BencodeValue::Bytes(file_key) =
                                                        &file_dict[k]
                                                    {
                                                        match file_key.as_ref() {
                                                            b"length" => {
                                                                if let BencodeValue::Integer(
                                                                    length,
                                                                ) = file_dict[k + 1]
                                                                {
                                                                    file_length = length as usize;
                                                                }
                                                            }
                                                            b"path" => {
                                                                if let BencodeValue::List(
                                                                    path_list,
                                                                ) = &file_dict[k + 1]
                                                                {
                                                                    for path_component in path_list
                                                                    {
                                                                        if let BencodeValue::Bytes(
                                                                            path_bytes,
                                                                        ) = path_component
                                                                        {
                                                                            let path_str = String::from_utf8(path_bytes.to_vec())
                                                                                .map_err(|e| anyhow!("Invalid UTF-8 in File path , Error parsing file {}" , e ))?;
                                                                            file_path
                                                                                .push(path_str);
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                    k += 2;
                                                }

                                                torrent_files.push(TorrentFile {
                                                    path: file_path,
                                                    length: file_length,
                                                });
                                            }
                                        }
                                        return Ok(Some(torrent_files));
                                    }
                                } else if files_key_bytes.as_ref() == b"length" {
                                    // Single file torrent
                                    return Ok(None);
                                }
                            }
                            j += 2;
                        }
                        // If we found info dict but no files field, it's a single-file torrent
                        return Ok(None);
                    }
                }
            }
            i += 2;
        }

        Err(anyhow!("Info field not found in dictionary"))
    }

    // Helper Functions
    fn encode_bencode(value: &BencodeValue, buf: &mut Vec<u8>) -> Result<()> {
        match value {
            BencodeValue::Integer(i) => {
                buf.extend_from_slice(b"i");
                buf.extend_from_slice(i.to_string().as_bytes());
                buf.extend_from_slice(b"e");
            }
            BencodeValue::List(ls) => {
                buf.extend_from_slice(b"l");
                for i in ls {
                    Self::encode_bencode(i, buf)?;
                }
                buf.extend_from_slice(b"e");
            }
            BencodeValue::Dictionary(dict) => {
                buf.extend_from_slice(b"d");
                let mut i = 0;
                while i + 1 < dict.len() {
                    let key = &dict[i];
                    let val = &dict[i + 1];
                    Self::encode_bencode(key, buf)?;
                    Self::encode_bencode(val, buf)?;
                    i += 2;
                }
                buf.extend_from_slice(b"e");
            }
            BencodeValue::Bytes(bytes) => {
                buf.extend_from_slice(bytes.len().to_string().as_bytes());
                buf.extend_from_slice(b":");
                buf.extend_from_slice(bytes);
            }
        }
        Ok(())
    }
}
