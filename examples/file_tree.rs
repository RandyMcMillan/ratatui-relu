//! File tree widget demo.
//!
//! Run with: cargo run --example file_tree

use std::io::{self, Stdout, stdout};
use std::path::PathBuf;
use std::time::Instant;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, widgets::Block};
use ratatui::style::Color;

use ratatui_relu::file_tree::{FileTreeState, FileTreeWidget};
use ratatui_relu::mouse::{disable_mouse_capture, enable_mouse_capture};

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    enable_mouse_capture()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_mouse_capture()?;
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let mut state = FileTreeState::from_paths(vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
        PathBuf::from("src/file_tree.rs"),
        PathBuf::from("examples/file_tree.rs"),
        PathBuf::from("Cargo.toml"),
    ]);
    let widget = FileTreeWidget::new().block(Block::bordered().title("Files"));
    let mut last_tick = Instant::now();
    loop {
        let _ = last_tick;
        last_tick = Instant::now();

        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(area);
            f.render_stateful_widget(widget.clone(), chunks[0], &mut state);
            let right = Block::bordered().title("Preview");
            f.render_widget(right, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    state.handle_key(key.code);
                    if matches!(key.code, crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc) {
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    restore_terminal(&mut terminal)?;
    let _ = Color::Reset;
    Ok(())
}
