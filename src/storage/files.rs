use crate::protocol::torrent::*;
use anyhow::anyhow;
use sha1::{Digest, Sha1};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct FileStorage {
    /// Base directory where files are stored
    pub download_dir: PathBuf,
    /// The torrent we're managing storage for
    pub torrent: Torrent,
    /// File mappings for multi-file torrents
    ///
    /// File mapping is simply mapping path routes for files in the torrent
    ///
    /// So eg lets say a torrent has a folder called 'media/james.png' this struct allows us to create a mapping for that '.png' file
    /// Whether the file is in a dir or not
    pub file_map: Vec<FileMapping>,
    /// Total length of a SINGLE FILE PAYLOAD / Size of a SINGLE FILE PAYLOAD (in mb/kb/gb)
    pub total_length: usize,
}

#[derive(Debug, Clone)]
/// Data representation of a single file , one that WILL be written to dics
pub struct FileMapping {
    /// Path to the file
    pub path: PathBuf,
    /// Starting byte offset in the torrent
    pub start_offset: usize,
    /// Length of this file
    pub length: usize,
    /// Whether the file exists and is complete
    pub is_complete: bool,
}

#[derive(Debug)]
pub struct PieceWrite {
    pub piece_index: usize,
    pub data: Vec<u8>,
}

impl FileStorage {
    pub fn from(torrent: Torrent, download_dir: PathBuf) -> Self {
        let file_map = Self::build_file_map(&torrent, &download_dir).unwrap_or_default();
        let total_length = torrent.length;

        let mut storage = FileStorage {
            download_dir,
            torrent,
            file_map,
            total_length,
        };

        // Create directory structure
        storage.create_directories().unwrap();

        // Check existing files
        storage.check_existing_files().unwrap();

        storage
    }

    fn build_file_map(
        torrent: &Torrent,
        download_dir: &Path,
    ) -> Result<Vec<FileMapping>, anyhow::Error> {
        let mut file_map = Vec::new();
        let mut current_offset = 0;

        match &torrent.files {
            Some(files) => {
                // Multi-file Torrent
                //
                // Creates the base directory for files and folders in the .torrent file
                let base_dir = download_dir.join(&torrent.name);
                // I decided to iterate over evry file and the make a path fr them
                for file in files {
                    let file_path = file
                        .path
                        .iter()
                        .fold(base_dir.clone(), |acc, part| acc.join(part));

                    file_map.push(FileMapping {
                        path: file_path,
                        start_offset: current_offset,
                        length: file.length,
                        is_complete: false,
                    });

                    current_offset += file.length;
                }
            }
            None => {
                // Single file torrent
                let file_path = download_dir.join(&torrent.name);
                file_map.push(FileMapping {
                    path: file_path,
                    start_offset: 0,
                    length: torrent.length,
                    is_complete: false,
                });
            }
        }

        Ok(file_map)
    }

    /// Creates directories for each and every file map in the file_map field
    fn create_directories(&self) -> Result<(), anyhow::Error> {
        for mapping in &self.file_map {
            // creates the parent dir of a single file
            if let Some(parent) = mapping.path.parent() {
                fs::create_dir_all(parent)?;
            } else {
                return Err(anyhow!("Theres an error"));
            }
        }

        Ok(())
    }

    /// Checks if files are complete
    pub fn check_existing_files(&mut self) -> Result<(), anyhow::Error> {
        for mapping in &mut self.file_map {
            if mapping.path.exists() {
                let metadata = fs::metadata(&mapping.path)?;

                if metadata.len() as usize == mapping.length {
                    mapping.is_complete = true;
                    println!("✅ Found complete file : {}", mapping.path.display());
                } else {
                    println!(
                        "⚠️  Found partial file: {} ({} bytes, expected {})",
                        mapping.path.display(),
                        metadata.len(),
                        mapping.length
                    );
                }
            }
        }
        Ok(())
    }

    /// How does this method work?
    ///
    /// It first verifies the hash of the piece at the specified index
    ///
    /// It then gets the start and end of the piece
    ///
    /// Finds files which the piece spans
    ///
    /// Writes into those files
    pub fn write_piece(&mut self, piece_index: usize, data: &[u8]) -> Result<(), anyhow::Error> {
        // Verify piece hash
        if !self.verify_piece_hash(piece_index, data)? {
            return Err(anyhow!("Piece {} hash verification failed", piece_index));
        }

        // start of the piece , usually the index * piece length
        let piece_start = piece_index * self.torrent.piece_length;
        let diff = (piece_start + data.len()).min(self.total_length);
        let piece_end = diff;
        let affected_files = self.get_affected_files(piece_start, piece_end)?;

        // Let's say the file starts at the beginning of the piece
        let mut offset = 0;
        for (file_mapping, file_start, file_end) in affected_files {
            // postion to start writing
            let write_start = file_start - piece_start;
            let write_length = file_end - file_start;
            let file_data = &data[offset..offset + write_length];
            self.write_to_file(&file_mapping.path, write_start, file_data)?;
            offset += write_length;
        }

        Ok(())
    }

    fn get_affected_files(
        &self,
        start: usize,
        end: usize,
    ) -> Result<Vec<(&FileMapping, usize, usize)>, anyhow::Error> {
        let mut affected = Vec::new();

        for mapping in &self.file_map {
            let file_start = mapping.start_offset;
            let file_end = mapping.start_offset + mapping.length;

            // Check if this file overlaps with the piece
            if start < file_end && end > file_start {
                let overlap_start = start.max(file_start);
                let overlap_end = end.min(file_end);
                affected.push((mapping, overlap_start, overlap_end));
            }
        }

        Ok(affected)
    }

    pub fn read_piece(&self, piece_index: usize) -> Result<Vec<u8>, anyhow::Error> {
        // Gets the piece start of the passed index
        // The start of a piece multiplied by the defined piece_length (eg 2 * 45kb = 90kb)
        let piece_start = piece_index * self.torrent.piece_length;

        // Usuallly the last piece of a file is smaller , so we check if a pieces index is equal to that of the last element and if so we get its actual size
        let piece_length = if piece_index == self.torrent.pieces.len() - 1 {
            // Last piece is usually shorter
            // eg (100kb - 90kb = 10kb)
            self.total_length - piece_start
        } else {
            // Simply return the piece length
            self.torrent.piece_length
        };

        // End of a piece (eg 19kb + 16kb = 35kb)
        let piece_end = piece_start + piece_length;
        // Offset to read the file from / Simply the position in the file to read from
        let mut offset = 0;

        // Buffer to hold the data
        let mut piece_data = vec![0u8; piece_length];
        // This gets a list of all files (and the overlapping byte ranges) that this piece belongs to.
        let affected_files = self.get_affected_files(piece_start, piece_end)?;

        for (file_mapping, file_start, file_end) in affected_files {
            let read_start = file_start - file_mapping.start_offset;
            let read_length = file_end - file_start;

            // Reads the file and we then push the data to our buffer
            let file_data = self.read_from_file(&file_mapping.path, read_start, read_length)?;
            piece_data[offset..offset + read_length].copy_from_slice(&file_data);
            offset += read_length;
        }

        Ok(piece_data)
    }

    #[doc = r"Simply reads a file

Offset is simple which index of the file to start from"]
    pub fn read_from_file(
        &self,
        path: &Path,
        offset: usize,
        length: usize,
    ) -> Result<Vec<u8>, anyhow::Error> {
        // opens the file
        let mut file = File::open(path)?;
        file.seek(SeekFrom::Start(offset as u64))?;

        let mut buffer = vec![0u8; length];
        file.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    /// Writes to a file
    pub fn write_to_file(
        &self,
        path: &Path,
        offset: usize,
        data: &[u8],
    ) -> Result<(), anyhow::Error> {
        let mut file = OpenOptions::new().create(true).write(true).open(path)?;

        file.seek(SeekFrom::Start(offset as u64))?;
        file.write_all(data)?;
        file.flush()?;

        Ok(())
    }

    /// Verifies a piece hash
    pub fn verify_piece_hash(
        &self,
        piece_index: usize,
        data: &[u8],
    ) -> Result<bool, anyhow::Error> {
        // Simply checks if the piece is actually part of the torrent
        if piece_index >= self.torrent.pieces.len() {
            return Err(anyhow!("Piece index {} out of range", piece_index));
        }

        // Hashes it
        let mut hasher = Sha1::new();
        hasher.update(data);
        let hash = hasher.finalize();

        let expected_hash = &self.torrent.pieces[piece_index];
        Ok(hash.as_slice() == expected_hash)
    }

    /// Checks if a piece is complete
    pub fn is_piece_complete(&self, piece_index: usize) -> Result<bool, anyhow::Error> {
        // Reads the piece
        match self.read_piece(piece_index) {
            // Verifies it
            Ok(data) => self.verify_piece_hash(piece_index, &data),
            Err(_) => Ok(false),
        }
    }

    pub fn get_completion_status(&self) -> Result<(usize, usize), anyhow::Error> {
        let mut complete_pieces = 0;
        let total_pieces = self.torrent.pieces.len();

        for i in 0..total_pieces {
            if self.is_piece_complete(i).unwrap_or(false) {
                complete_pieces += 1;
            }
        }

        Ok((complete_pieces, total_pieces))
    }

    /// Gets the missing pieces
    pub fn get_missing_pieces(&self) -> Result<Vec<usize>, anyhow::Error> {
        let mut missing = Vec::new();

        for i in 0..self.torrent.pieces.len() {
            if !self.is_piece_complete(i).unwrap_or(false) {
                missing.push(i);
            }
        }

        Ok(missing)
    }

    pub fn get_total_bytes(&self) -> usize {
        self.total_length
    }

    pub fn get_download_dir(&self) -> &std::path::Path {
        &self.download_dir
    }
}
