//! Smoothly animates the sidebar open and closed using a continuous ReLU gate.
//!
//! Run in a real terminal with: cargo run --example animation

use std::io::{self, stdout, Stdout};
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use ratarelu::animation::Transition;
use ratarelu::layout::handwired_layout_engine;

struct App {
    toggle: Transition,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            toggle: Transition::new(0.0, 3.0),
            should_quit: false,
        }
    }

    fn toggle(&mut self) {
        self.toggle.toggle_between(0.0, 1.0);
    }

    fn update(&mut self, dt: Duration) {
        self.toggle.update(dt);
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

        if event::poll(Duration::from_millis(16))? && let Event::Key(key) = event::read()? && key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                KeyCode::Char('t') | KeyCode::Char(' ') => app.toggle(),
                _ => {}
            }
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();
    let current = app.toggle.current();
    let target = app.toggle.target();

    let (layout, gate) = handwired_layout_engine(current, area.width);

    let text = Text::from(vec![
        Line::from(vec![
            Span::raw("Animated Sidebar Demo | current: "),
            Span::styled(
                format!("{:.3}", current),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" target: "),
            Span::styled(
                format!("{:.1}", target),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(format!(" | gate: {:.3}", gate)),
        ]),
        Line::from("Press [t]/[Space] to animate, [q]/[Esc] to quit."),
    ]);

    let header = Paragraph::new(text)
        .block(Block::default().title("Animation").borders(Borders::ALL));

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    f.render_widget(header, vertical[0]);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(layout.sidebar_width), Constraint::Min(0)])
        .split(vertical[1]);

    let sidebar = Paragraph::new(format!("Sidebar\n{} cols", layout.sidebar_width))
        .block(Block::default().title("Sidebar").borders(Borders::ALL))
        .alignment(Alignment::Center);

    let main = Paragraph::new(format!("Main\n{} cols", layout.main_width))
        .block(Block::default().title("Main").borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(sidebar, horizontal[0]);
    f.render_widget(main, horizontal[1]);
}
