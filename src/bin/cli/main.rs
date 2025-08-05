use clap::Parser;
use ratatui::{DefaultTerminal, Frame};

#[derive(Parser, Debug)]
struct TorrentArgs {
    path: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("WORKING");
    Ok(())
}
