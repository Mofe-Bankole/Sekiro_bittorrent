use anyhow::{Result, anyhow};
use sha1::{Digest, Sha1};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Standard BitTorrent block size (16KB)
pub const BLOCK_SIZE: usize = 16 * 1024;

/// Maximum number of pending requests per peer
pub const MAX_PENDING_REQUESTS: usize = 10;

/// Request timeout duration
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]

// Information that is usually
pub struct BlockInfo {
    pub piece_index: usize,
    // Where this block begins
    pub begin: usize,
    // Size of the block in bytes
    pub length: usize,
}

impl BlockInfo {
    pub fn new(piece_index: usize, begin: usize, length: usize) -> Self {
        Self {
            piece_index,
            begin,
            length,
        }
    }
}

/// Data representation of a Block
///
/// A block is a smaller piece of a piece
#[derive(Debug, Clone)]
pub struct Block {
    pub info: BlockInfo,
    pub data: Vec<u8>,
    pub received_at: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PieceState {
    /// Not started downloading
    Pending,
    /// Currently being downloaded
    InProgress,
    /// All blocks downloaded, awaiting verification
    Complete,
    /// Verified and written to disk
    Verified,
    /// Failed hash verification
    Failed,
}

#[derive(Debug)]
/// Data representation of a Piece in a torrent
pub struct Piece {
    /// Index of a piece
    pub index: usize,
    /// The pieces length or size (mb or bytes for simplicity)
    pub length: usize,
    // The pieces hash
    pub hash: [u8; 20],
    // The pieces state
    pub state: PieceState,

    //Block tracking
    pub blocks: HashMap<usize, Block>,
    pub missing_blocks: HashSet<BlockInfo>,
    pub requested_blocks: HashMap<BlockInfo, Instant>,

    // Timing
    pub download_start: Option<Instant>,
    pub download_complete: Option<Instant>,
}

impl Piece {
    pub fn new(index: usize, length: usize, hash: [u8; 20]) -> Self {
        let num_blocks = (length + BLOCK_SIZE - 1) / BLOCK_SIZE;
        let mut missing_blocks = HashSet::new();

        for i in 0..num_blocks {
            let begin = i * BLOCK_SIZE;
            let block_length = if i == num_blocks - 1 {
                length - begin
            } else {
                BLOCK_SIZE
            };

            missing_blocks.insert(BlockInfo::new(index, begin, block_length));
        }

        Self {
            index,
            length,
            hash,
            state: PieceState::Pending,
            blocks: HashMap::new(),
            missing_blocks,
            requested_blocks: HashMap::new(),
            download_start: None,
            download_complete: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.missing_blocks.is_empty() && self.blocks.len() * BLOCK_SIZE >= self.length
    }

    pub fn get_next_block_request(&mut self) -> Option<BlockInfo> {
        // Clean up timeouts
        let now = Instant::now();

        // Gets timedout blocks
        let timed_out: Vec<BlockInfo> = self
            .requested_blocks
            .iter()
            .filter(|&(_, &time)| now.duration_since(time) > REQUEST_TIMEOUT)
            .map(|(block, _)| *block)
            .collect();

        for block in timed_out {
            self.requested_blocks.remove(&block);
            // Add to the missing blocks
            self.missing_blocks.insert(block);
        }

        if let Some(&block) = self.missing_blocks.iter().next() {
            if self.requested_blocks.len() < MAX_PENDING_REQUESTS {
                self.missing_blocks.remove(&block);
                self.requested_blocks.insert(block, now);
                return Some(block);
            }
        }
        None
    }

    pub fn add_block(&mut self, block: Block) -> Result<()> {
        // Validate block
        // Makes sure the blocks parent PIECE is the PIECE
        if block.info.piece_index != self.index {
            return Err(anyhow!("Block index is not equal to pieces index"));
        }

        if block.info.begin + block.data.len() > self.length {
            return Err(anyhow!("Block exceeds Piece Size"));
        }

        self.requested_blocks.remove(&block.info);
        self.blocks.insert(block.info.begin, block);

        self.download_start = Some(Instant::now());

        if self.is_complete() {
            self.download_complete = Some(Instant::now());
            self.state = PieceState::Complete;
        }

        Ok(())
    }

    pub fn assemble_piece(&self) -> Result<Vec<u8>> {
        if !self.is_complete() {
            return Err(anyhow!("Piece is not Complete"));
        }

        let mut piece_data = vec![0u8; self.length];

        for (begin, block) in &self.blocks {
            // begin in this case is most times zero ( 0 )
            // end is definetely the blocks size in this case
            // blocks begin is added to the blocks size eg 1 + 47
            let end = begin + block.data.len();

            if end > self.length {
                return Err(anyhow!("Block exceeds piece length"));
            }

            piece_data[*begin..end].copy_from_slice(&block.data);
        }

        Ok(piece_data)
    }

    pub fn verify_hash(&self, data: &[u8]) -> bool {
        let mut hasher = Sha1::new();
        hasher.update(data);
        let hash = hasher.finalize();
        hash.as_slice() == &self.hash
    }

    pub fn reset(&mut self) {
        self.state = PieceState::Pending;
        self.blocks.clear();
        self.requested_blocks.clear();
        self.download_start = None;
        self.download_complete = None;

        // Rebuild missing blocks
        let num_blocks = (self.length + BLOCK_SIZE - 1) / BLOCK_SIZE;
        self.missing_blocks.clear();

        for i in 0..num_blocks {
            let begin = i * BLOCK_SIZE;
            let block_length = if i == num_blocks - 1 {
                self.length - begin
            } else {
                BLOCK_SIZE
            };
            self.missing_blocks
                .insert(BlockInfo::new(self.index, begin, block_length));
        }
    }
}
