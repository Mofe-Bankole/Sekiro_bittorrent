use clap::Parser;
use std::error::Error;
use torrex::bencode::Torrent;

#[derive(Parser, Debug)]
struct Args {
    #[args(short, long)]
    torrent: path::PathBuf,
}

async fn main() -> Result<(), Box<dyn Error>> {
    tokio::spawn(async {}).await?;

    fn run() {
        println!("Hello, world!");
    }
    Ok(())
}
