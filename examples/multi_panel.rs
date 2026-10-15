//! Multi-panel layout controlled by independent animated transitions.
//!
//! Each panel uses a [`Transition`] to smoothly open and close instead of
//! snapping instantly.
//!
//! Run in a real terminal with: cargo run --example multi_panel

use std::io::{self, stdout, Stdout};
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use ratarelu::animation::Transition;

struct App {
    header: Transition,
    sidebar: Transition,
    footer: Transition,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            // Start all panels open.
            header: Transition::new(1.0, 4.0),
            sidebar: Transition::new(1.0, 4.0),
            footer: Transition::new(1.0, 4.0),
            should_quit: false,
        }
    }

    fn update(&mut self, dt: Duration) {
        self.header.update(dt);
        self.sidebar.update(dt);
        self.footer.update(dt);
    }
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    while !app.should_quit {
        let now = Instant::now();
        let dt = now - last_tick;
        last_tick = now;

        app.update(dt);
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                KeyCode::Char('h') => app.header.toggle_between(0.0, 1.0),
                KeyCode::Char('s') => app.sidebar.toggle_between(0.0, 1.0),
                KeyCode::Char('f') => app.footer.toggle_between(0.0, 1.0),
                KeyCode::Char(' ') => {
                    app.header.toggle_between(0.0, 1.0);
                    app.sidebar.toggle_between(0.0, 1.0);
                    app.footer.toggle_between(0.0, 1.0);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();

    let header_height = (app.header.current() * 3.0).round() as u16;
    let footer_height = (app.footer.current() * 3.0).round() as u16;
    let sidebar_width = (app.sidebar.current() * 30.0).round() as u16;

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(0),
            Constraint::Length(footer_height),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(sidebar_width), Constraint::Min(0)])
        .split(vertical[1]);

    render_panel(f, "Header", vertical[0], Color::Blue);
    render_panel(f, "Sidebar", horizontal[0], Color::Green);
    render_panel(f, "Main", horizontal[1], Color::Reset);
    render_panel(f, "Footer", vertical[2], Color::Yellow);

    let help = Paragraph::new("[h] header  [s] sidebar  [f] footer  [Space] all  [q] quit")
        .alignment(Alignment::Center);
    let help_area = Rect::new(
        horizontal[1].x + 2,
        horizontal[1].y + horizontal[1].height.saturating_sub(2) / 2,
        horizontal[1].width.saturating_sub(4),
        1,
    );
    f.render_widget(help, help_area);
}

fn render_panel(f: &mut Frame, title: &str, area: Rect, color: Color) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width > 0 && inner.height > 0 {
        let text = Paragraph::new(format!("{} area", title)).alignment(Alignment::Center);
        f.render_widget(text, inner);
    }
}
