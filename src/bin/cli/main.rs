use crate::block_manager::{BlockData, BlockInfo, BlockManager};
use clap::Parser;
use color_eyre::Result;
use mini_p2p_file_transfer_system::{
    net::download_manager::BlockManager, protocol::torrent::Torrent, storage::files::FileStorage,
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    prelude::*,
    widgets::Paragraph,
};
use std::{fmt::format, fs, path::PathBuf, vec};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE", help = "Path to the .torrent file")]
    path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct App {
    pub path: PathBuf,
    pub app_name: String,

    /// Whether the app should exit
    pub should_quit: bool,
    pub selected_index: usize,
    pub torrent: Option<Torrent>,
    pub file_storage: Option<FileStorage>,
    pub download_dir: PathBuf,
    pub block_manager: Option<BlockManager>,
    pub error_message: Option<String>,
}

impl App {
    pub fn new(path: PathBuf, app_name: String) -> Self {
        Self {
            path,
            app_name,
            should_quit: false,
            selected_index: 0,
            torrent: None,
            error_message: None,
            download_dir: PathBuf::from("~/Downloads"),
            file_storage: FileStorage,
            block_manager: BlockManager,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn next(&mut self) {
        self.selected_index = self.selected_index.saturating_add(1);
    }

    pub fn previous(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn handle_key_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('p') => self.previous(),
            KeyCode::Char('n') => self.next(),
            KeyCode::Esc => self.quit(),
            _ => {}
        }
    }

    pub fn load_torrent(&mut self) {
        // Checks if the path exists
        if !self.path.exists() {
            self.error_message = Some(format!(
                "Torrent Filed Does not exist : {}",
                self.path.display()
            ));
            self.torrent = None;
            return;
        }

        // Checks if its actually a file
        if !self.path.is_file() {
            self.error_message = Some(format!(
                "
                {} Torrent is not a file",
                self.path.display()
            ));
            self.torrent = None;
            return;
        }

        match fs::read(&self.path) {
            // Converts the READ file to a Torrent
            Ok(bytes) => match Torrent::from_bytes(&bytes) {
                Ok(torrent) => {
                    self.torrent = Some(torrent);
                    self.error_message = None;

                    match FileStorage::new(torrent.clone(), self.download_dir) {
                        Ok(storage) => match BlockManager::new(torrent.clone(), storage) {
                            Ok(manager) => {
                                self.block_manager = manager;
                                self.error_message = None;
                            }
                            Err(e) => {
                                self.error_message =
                                    Some(format!("Failed to init block manager: {}", e));
                            }
                        },
                        Err(e) => {
                            self.error_message =
                                Some(format!("File Storage could not be built : {}", e))
                        }
                    }
                }
                Err(e) => {
                    self.torrent = None;
                    self.error_message = Some(format!("Torrent File could not be found : {}", e))
                }
            },
            Err(e) => {
                self.error_message = Some(format!("Failed to parse torrent: {}", e));
                self.torrent = None;
            }
        }
    }

    pub fn simulate_download_step(&mut self) {
        if let Some(manager) = &mut self.block_manager {
            // Get next piece to work on
            if let Some(piece_index) = manager.get_next_piece_to_download() {
                // Get all blocks for this piece
                loop {
                    match manager.get_next_block_request(piece_index) {
                        Some(block_info) => {
                            // Simulate receiving block data
                            // In real app, this comes from network
                            let dummy_data = vec![0u8; block_info.length];

                            let block_data = BlockData {
                                info: block_info,
                                data: dummy_data,
                                received_at: std::time::Instant::now(),
                            };

                            // Process the block
                            match manager.handle_block_received(block_data) {
                                Ok(_) => {
                                    // Block processed successfully
                                }
                                Err(e) => {
                                    self.error_message = Some(format!("Block error: {}", e));
                                    break;
                                }
                            }
                        }
                        None => {
                            // No more blocks for this piece
                            break;
                        }
                    }
                }

                // Update status
                let stats = manager.get_stats();
                self.status_message = Some(format!(
                    "Downloaded piece {}. Progress: {:.1}%",
                    piece_index,
                    stats.progress_percentage()
                ));
            } else {
                self.status_message = Some("Download complete!".to_string());
            }
        } else {
            self.error_message = Some("No block manager initialized".to_string());
        }
    }

    pub fn show_stats(&mut self) {
        if let Some(manager) = &self.block_manager {
            let stats = manager.get_stats();

            let message = format!(
                "Progress: {}/{} pieces ({:.1}%)\n\
                Downloaded: {} / {} bytes\n\
                Speed: {:.2} KB/s\n\
                Missing: {} pieces",
                stats.verified_pieces,
                stats.total_pieces,
                stats.progress_percentage(),
                stats.downloaded_bytes,
                stats.total_bytes,
                stats.download_speed_bps() / 1024.0,
                manager.get_missing_piece_count()
            );

            self.status_message = Some(message);
        }
    }
    pub fn execute_selected_action(&mut self) {
        match self.selected_index {
            0 => self.view_torrent_data(),
            1 => self.view_peers(),
            2 => self.quit(),
        }
    }

    fn view_peers(&self) {
        todo!()
    }

    fn view_torrent_data(&self) {
        todo!()
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.path.unwrap_or_else(|| {
        eprintln!("Path not provided, using current directory");
        PathBuf::from("./test.torrent")
    });

    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = App::new(path, "BitTorrent Clone".to_string());
    app.load_torrent();
    let result = run(terminal, app);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|frame| render(frame, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key_input(key.code);
            }

            if app.should_quit {
                eprintln!("App has shutdown");
                break;
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame, app: &App) {
    let mut content = String::new();

    content.push_str("========== BitTorrent Clone ==========\n");
    content.push_str(&format!("Torrent: {}\n", app.path.display()));
    content.push_str(&format!("Download Dir: {}\n\n", app.download_dir.display()));

    content.push_str("Controls:\n");
    content.push_str("  q/Esc: Quit\n");
    content.push_str("  r: Reload torrent\n");
    content.push_str("  d: Download next piece\n");
    content.push_str("  s: Show statistics\n\n");

    if let Some(error) = &app.error_message {
        content.push_str(&format!("ERROR: {}\n\n", error));
    }

    if let Some(status) = &app.status_message {
        content.push_str(&format!("STATUS: {}\n\n", status));
    }

    // Show torrent info
    if let Some(torrent) = &app.torrent {
        content.push_str(&format!(
            "Torrent Info:\n\
            - Name: {}\n\
            - Size: {} bytes\n\
            - Pieces: {}\n\
            - Piece Length: {} bytes\n\n",
            torrent.name,
            torrent.length,
            torrent.pieces.len(),
            torrent.piece_length
        ));
    }

    // Show download progress
    if let Some(manager) = &app.block_manager {
        let stats = manager.get_stats();

        content.push_str("Download Progress:\n");

        // Progress bar
        let bar_width = 40;
        let filled = if stats.total_pieces > 0 {
            (stats.verified_pieces * bar_width) / stats.total_pieces
        } else {
            0
        };
        let bar = "=".repeat(filled) + &"-".repeat(bar_width - filled);
        content.push_str(&format!(
            "[{}] {:.1}%\n\n",
            bar,
            stats.progress_percentage()
        ));

        content.push_str(&format!(
            "Pieces: {}/{}\n\
            Bytes: {}/{}\n",
            stats.verified_pieces, stats.total_pieces, stats.downloaded_bytes, stats.total_bytes
        ));
    }

    let text = Paragraph::new(content).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(text, frame.area());
}
