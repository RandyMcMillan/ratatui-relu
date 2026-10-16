//! Single side-menu panel that opens on start.
//!
//! Demonstrates a left sidebar that is open when the app starts and can be
//! toggled with the arrow keys.
//!
//! Run in a real terminal with: cargo run --example side_menu_panel

use std::io::{self, Stdout, stdout};
use std::time::{Duration, Instant};

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use ratatui_relu::animation::Transition;
use ratatui_relu::layout::handwired_layout_engine;

struct App {
    sidebar: Transition,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            // The side menu's initial state is set here.
            // `Transition::new(initial_value, speed)` takes the starting value as
            // its first argument. Use `1.0` for fully open and `0.0` for fully
            // closed. The second argument is the animation speed in units/sec.
            sidebar: Transition::new(1.0, 4.0),
            should_quit: false,
        }
    }

    fn update(&mut self, dt: Duration) {
        self.sidebar.update(dt);
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
                KeyCode::Char(' ') | KeyCode::Left | KeyCode::Right => {
                    app.sidebar.toggle_between(0.0, 1.0);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();
    let current = app.sidebar.current();

    let (layout, gate) = handwired_layout_engine(current, area.width);

    let text = Text::from(vec![
        Line::from(vec![
            Span::raw("Side Menu Panel Demo | sidebar: "),
            Span::styled(
                format!("{:.1}", current),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" | gate: {:.3}", gate)),
        ]),
        Line::from("Press [Space], [←], or [→] to toggle the side menu. [q] or [Esc] to quit."),
    ]);

    let header =
        Paragraph::new(text).block(Block::default().title("Side Menu").borders(Borders::ALL));

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    f.render_widget(header, vertical[0]);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(layout.sidebar_width), Constraint::Min(0)])
        .split(vertical[1]);

    let sidebar = Paragraph::new(format!("Menu\n{} cols", layout.sidebar_width))
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .alignment(Alignment::Center);

    let main = Paragraph::new(format!("Main\n{} cols", layout.main_width))
        .block(Block::default().title("Main").borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(sidebar, horizontal[0]);
    f.render_widget(main, horizontal[1]);
}
