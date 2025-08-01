use clap::Parser;
use std::error::Error;
use torrex::bencode::Torrent;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame};

#[derive(Parser, Debug)]
struct TorrentArgs{
    path : PathBuf,
}

async fn main() -> Result<(), Box<dyn Error>> {
    tokio::spawn(async move {
        color_eyre::install()?;
        let args = TorrentArgs::parse();
        let torrent = Torrent::from_file(&args.path)?;
        println!("Torrent: {:?}", torrent);
    }).await?;

    Ok(())
}

