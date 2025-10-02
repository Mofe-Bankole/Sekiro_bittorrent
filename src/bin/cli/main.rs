use clap::Parser;
use color_eyre::Result;
use mini_p2p_file_transfer_system::protocol::torrent::Torrent;
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
    pub download_dir : PathBuf
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
            download_dir : PathBuf::from("~/Downloads")
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
            // saves it if its an actual torrent
            Ok(bytes) => match Torrent::from_bytes(&bytes) {
                Ok(torrent) => {
                    self.torrent = Some(torrent);
                    self.error_message = None
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

    pub fn execute_selected_action(&mut self){
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
    let options = vec!["View Torrent Data", "View Peers", "Exit Program"];

    content.push_str( "----------BitTorrent Clone - Press 'q' or 'Esc' to quit--------\nPath: {}\nSelected Index: {}");
    content.push_str(&format!("Torrent files : {}\n", app.path.display()));
    content.push_str(&format!("Download dir : {}\n", app.download_dir.display()));
    content.push_str(&format!("Selected Index: {}\n", app.selected_index));
    content.push_str("Controls: q=Quit, p/n=Previous/Next, r=Reload\n\n");

    if let Some(error) = &app.error_message {
        content.push_str(&format!("ERROR: {}\n\n", error));
    }

    for (i , option) in options.iter().enumerate(){
        if i == app.selected_index {
            content.push_str(&format!("> {} <\n", option)); // Highlight selected
        } else {
            content.push_str(&format!("  {}\n", option));
        }
    }
    frame.render_widget(content, frame.area());
}
