use crate::{
    net::piece_manager::{BLOCK_SIZE, Block, BlockInfo, Piece, PieceState},
    protocol::torrent::Torrent,
    storage::files::FileStorage,
};
use anyhow::anyhow;
use std::{
    collections::VecDeque,
    io::Error,
    sync::{Arc, Mutex},
    time::Instant,
};

#[derive(Debug)]
pub struct BlockManager {
    torrent: Torrent,
    pieces: Vec<Arc<Mutex<Piece>>>,
    storage: Arc<Mutex<FileStorage>>,
    download_queue: VecDeque<usize>,
    stats: DownloadStats,
}

#[derive(Debug, Clone, Default)]
pub struct DownloadStats {
    // Total pieces of a torrent to be downloaded
    pub total_pieces: usize,
    // Number of completed pieces
    pub completed_pieces: usize,
    pub verified_pieces: usize,
    pub failed_pieces: usize,
    pub total_bytes: usize,
    pub downloaded_bytes: usize,
    pub download_start: Option<Instant>,
    pub last_update: Option<Instant>,
}

impl DownloadStats {
    pub fn progress_percentage(&self) -> f64 {
        if self.total_pieces == 0 {
            return 0.0;
        }
        (self.verified_pieces as f64 / self.total_pieces as f64) * 100.0
    }

    /// Gets download speed in bytes per seconds (bps)
    pub fn download_speed_bps(&self) -> f64 {
        if let (Some(start), Some(last)) = (self.download_start, self.last_update) {
            let elapsed = last.duration_since(start).as_secs_f64();
            if elapsed > 0.0 {
                return self.downloaded_bytes as f64 / elapsed;
            }
        }
        0.0
    }

    pub fn eta_seconds(&self) -> Option<u64> {
        let remaining = self.total_bytes.saturating_sub(self.downloaded_bytes);
        let speed = self.download_speed_bps();

        if speed > 0.0 && remaining > 0 {
            Some((remaining as f64 / speed) as u64)
        } else {
            None
        }
    }
}

impl BlockManager {
    pub fn new(torrent: Torrent, storage: FileStorage) -> Result<Self, Error> {
        let mut pieces = Vec::new();
        let piece_length = torrent.piece_length;
        let total_length = torrent.length;

        for (index, &hash) in torrent.pieces.iter().enumerate() {
            let length = if index == torrent.pieces.len() - 1 {
                total_length - (index * piece_length)
            } else {
                piece_length
            };

            pieces.push(Arc::new(Mutex::new(Piece::new(index, length, hash))));
        }

        let stats = DownloadStats {
            total_pieces: pieces.len(),
            total_bytes: total_length,
            ..Default::default()
        };

        let mut manager = Self {
            torrent,
            pieces,
            storage: Arc::new(Mutex::new(storage)),
            download_queue: VecDeque::new(),
            stats,
        };

        // Initialize download queue with missing pieces
        match manager.rebuild_download_queue() {
            Ok(_) => println!("Download Queue rebuilt"),
            Err(_) => println!("Error"),
        };

        Ok(manager)
    }

    pub fn rebuild_download_queue(&mut self) -> Result<(), anyhow::Error> {
        self.download_queue.clear();

        // Check which pieces we already have
        let storage = self.storage.lock().unwrap();
        for (index, piece_arc) in self.pieces.iter().enumerate() {
            let mut piece = piece_arc.lock().unwrap();

            if storage.is_piece_complete(index).unwrap_or(false) {
                piece.state = PieceState::Verified;
                self.stats.verified_pieces += 1;
                self.stats.downloaded_bytes += piece.length;
            } else {
                self.download_queue.push_back(index);
            }
        }

        Ok(())
    }

    /// Simple sequential strategy
    pub fn get_next_piece_to_download(&mut self) -> Option<usize> {
        self.download_queue.pop_front()
    }

    /// Gets the next block request , params are the blocks piece_index
    pub fn get_next_block_request(&self, piece_index: usize) -> Option<BlockInfo> {
        if piece_index >= self.pieces.len() {
            return None;
        }

        let mut piece = self.pieces[piece_index].lock().unwrap();

        if piece.state == PieceState::Pending {
            piece.state = PieceState::InProgress;
        }

        piece.get_next_block_request()
    }

    pub fn handle_block_received(&mut self, block: Block) -> Result<(), anyhow::Error> {
        // Gets the index of the block received
        let piece_index = block.info.piece_index;

        // Makes sure the blocks index is not greater than the len of pieces (i.e The Size of the piece)
        if piece_index > self.pieces.len() {
            return Err(anyhow!(
                "Invalid piece index: {}/nExceeds the pieces length",
                piece_index
            ));
        }

        // Find the piece in the Block Managers pieces
        let piece_arc = self.pieces[piece_index].clone();
        let mut piece = piece_arc.lock().unwrap();

        // Adds the block to its Parent Piece
        piece.add_block(block)?;

        // Sets the size of the block
        self.stats.downloaded_bytes = piece.blocks.len() * BLOCK_SIZE;
        self.stats.last_update = Some(Instant::now());

        if piece.state == PieceState::Complete {
            drop(piece); // Release lock before verification
            self.verify_and_write_piece(piece_index)?;
        }

        Ok(())
    }

    /// Verifies and writes a piece to storage
    pub fn verify_and_write_piece(&mut self, piece_index: usize) -> Result<(), anyhow::Error> {
        // Makes sure the pieces index is a part of the block managers pieces available
        let piece_arc = self.pieces[piece_index].clone();
        let mut piece = piece_arc.lock().unwrap();

        if piece.state != PieceState::Complete {
            return Err(anyhow!("Piece {} is not complete", piece_index));
        }

        // Assemble piece data
        let piece_data = piece.assemble_piece()?;

        // Verify hash
        if !piece.verify_hash(&piece_data) {
            eprintln!("Piece {} failed hash verification, resetting", piece_index);
            piece.state = PieceState::Failed;
            self.stats.failed_pieces += 1;

            // Reset piece for re-download
            piece.reset();
            self.download_queue.push_back(piece_index);

            return Err(anyhow!(
                "Hash verification failed for piece {}",
                piece_index
            ));
        }

        // Write to disk
        let mut storage = self.storage.lock().unwrap();
        storage.write_piece(piece_index, &piece_data)?;

        // Update state
        piece.state = PieceState::Verified;
        self.stats.completed_pieces += 1;
        self.stats.verified_pieces += 1;

        println!(
            "Piece {}/{} verified and written ({:.2}%)",
            piece_index + 1,
            self.pieces.len(),
            self.stats.progress_percentage()
        );

        Ok(())
    }

    pub fn get_piece_state(&self, piece_index: usize) -> Option<PieceState> {
        if piece_index >= self.pieces.len() {
            return None;
        }

        let piece = self.pieces[piece_index].lock().unwrap();
        Some(piece.state.clone())
    }

    pub fn get_stats(&self) -> DownloadStats {
        self.stats.clone()
    }

    pub fn is_download_complete(&self) -> bool {
        self.stats.verified_pieces == self.stats.total_pieces
    }

    pub fn has_piece(&self, piece_index: usize) -> bool {
        if piece_index >= self.pieces.len() {
            return false;
        }

        let piece = self.pieces[piece_index].lock().unwrap();
        piece.state == PieceState::Verified
    }

    pub fn get_missing_piece_count(&self) -> usize {
        self.stats.total_pieces - self.stats.verified_pieces
    }
}
