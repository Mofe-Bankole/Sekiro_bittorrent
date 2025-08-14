use color_eyre::{Result, eyre::Ok};
use ratatui::{DefaultTerminal, Frame, Terminal, prelude::Backend};

#[derive(Parser, Debug)]
struct App {
    path: Vec<String>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    Ok(result)
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}
fn render(frame: &mut Frame) {
    frame.render_widget("hello world", frame.area());
}
