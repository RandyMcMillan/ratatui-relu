//! Multi-panel layout controlled by independent animated transitions.
//!
//! Demonstrates the reusable [`MultiPanelApp`] from `ratatui_relu::multi_panel`.
//!
//! Run in a real terminal with: cargo run

use std::io::{self, Stdout, stdout};
use std::time::Instant;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use ratatui_relu::{
    mouse::{disable_mouse_capture, enable_mouse_capture},
    multi_panel::MultiPanelApp,
};

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
    let mut app = MultiPanelApp::new();
    let result = run_app(&mut terminal, &mut app);
    // Persist the panel state on exit. Ignore save errors so the terminal is
    // always restored cleanly.
    let _ = app.save();
    restore_terminal(&mut terminal)?;
    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut MultiPanelApp,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        let now = Instant::now();
        let dt = now - last_tick;
        last_tick = now;

        app.update(dt);
        terminal.draw(|f| app.render(f))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    app.handle_key_event(key);
                    if matches!(
                        key.code,
                        crossterm::event::KeyCode::Char('q')
                            | crossterm::event::KeyCode::Esc
                    ) {
                        break;
                    }
                    // Repaint immediately after a key press so state changes
                    // are visible right away.
                    terminal.draw(|f| app.render(f))?;
                }
                Event::Mouse(mouse) => {
                    app.handle_mouse(mouse);
                }
                _ => {}
            }
        }
    }
    Ok(())
}
