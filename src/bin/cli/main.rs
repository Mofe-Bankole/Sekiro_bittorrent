use clap::Parser;
use std::error::Error;
use torrex::bencode::Torrent;

#[derive(Parser, Debug)]
struct Args {
    #[args(short, long)]
    torrent: path::PathBuf,
}
async fn main() -> Result<(), Box<dyn Error>> {
    tokio::spawn(async {
        let args = Args::parse();
        if (!args) {
            return;
        }

        let torrent_path = args.torrent;

        let mut torrent = Torrent::new(torrent_path).await?;
        torrent.start().await?;
        torrent.parse_to_torrent(torrent_path).await?;
    })
    .await?;

    Ok(())
}
