//! Handwired Ratatui Layout Example in Rust
//! Demonstrates controlling TUI panel widths, visibility, and constraints
//! using continuous ReLU gates instead of standard conditional branching.

use std::io::{self, Stdout, stdout};

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

mod layout;
use layout::{handwired_layout_engine, standard_layout_engine};

struct App {
    toggle: f64,
    sidebar_open: bool,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            toggle: 0.0,
            sidebar_open: false,
            should_quit: false,
        }
    }

    fn toggle_sidebar(&mut self) {
        self.sidebar_open = !self.sidebar_open;
        self.toggle = if self.sidebar_open { 1.0 } else { 0.0 };
    }
}

fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    result
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

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                KeyCode::Char('t') | KeyCode::Char(' ') => app.toggle_sidebar(),
                _ => {}
            }
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let total_area = f.area();

    // Outer layout: header + body + footer
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(total_area);

    let header_area = vertical[0];
    let body_area = vertical[1];
    let footer_area = vertical[2];

    render_header(f, app, header_area);
    render_body(f, app, body_area);
    render_footer(f, footer_area);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let status = if app.sidebar_open {
        "Sidebar OPEN"
    } else {
        "Sidebar CLOSED"
    };
    let text = Text::from(vec![
        Line::from(vec![
            Span::raw("Handwired Ratatui Layout Demo | Toggle: "),
            Span::styled(
                format!("{:.1}", app.toggle),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(status, Style::default().fg(Color::Yellow)),
        ]),
        Line::from("Comparing standard conditional layout vs. handwired ReLU-gated layout."),
    ]);

    let header = Paragraph::new(text).block(
        Block::default()
            .title("RataReLU")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL),
    );
    f.render_widget(header, area);
}

fn render_body(f: &mut Frame, app: &App, area: Rect) {
    let total_width = area.width;
    let total_height = area.height;

    let std_layout = standard_layout_engine(app.toggle, total_width);
    let (hw_layout, gate) = handwired_layout_engine(app.toggle, total_width);

    // Standard engine result rendered as a real Ratatui layout
    let std_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(std_layout.sidebar_width),
            Constraint::Min(0),
        ])
        .split(Rect::new(area.x, area.y, total_width, total_height / 2));

    render_panel(
        f,
        "Standard Engine",
        std_layout.sidebar_width,
        std_layout.main_width,
        app.toggle,
        None,
        std_chunks,
    );

    // Handwired engine result rendered as a real Ratatui layout
    let hw_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(hw_layout.sidebar_width),
            Constraint::Min(0),
        ])
        .split(Rect::new(
            area.x,
            area.y + total_height / 2,
            total_width,
            total_height - total_height / 2,
        ));

    render_panel(
        f,
        "Handwired Engine",
        hw_layout.sidebar_width,
        hw_layout.main_width,
        app.toggle,
        Some(gate),
        hw_chunks,
    );
}

fn render_panel(
    f: &mut Frame,
    title: &str,
    sidebar_width: u16,
    main_width: u16,
    toggle: f64,
    gate: Option<f64>,
    chunks: std::rc::Rc<[Rect]>,
) {
    let sidebar = Paragraph::new(format!("Sidebar\nwidth: {}", sidebar_width))
        .block(Block::default().title("Sidebar").borders(Borders::ALL))
        .alignment(Alignment::Center);

    let mut main_text = vec![
        Line::from(format!("Engine: {}", title)),
        Line::from(format!("Toggle input: {:.1}", toggle)),
        Line::from(format!("Sidebar width: {} cols", sidebar_width)),
        Line::from(format!("Main width: {} cols", main_width)),
    ];
    if let Some(g) = gate {
        main_text.push(Line::from(format!("ReLU gate: {:.1}", g)));
    }

    let main = Paragraph::new(Text::from(main_text))
        .block(Block::default().title("Main").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(sidebar, chunks[0]);
    f.render_widget(main, chunks[1]);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new("Press [t] or [Space] to toggle sidebar  |  [q] or [Esc] to quit")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(footer, area);
}

// ==========================================
// UNIT TEST MODULE
// ==========================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handwired_layout_match() {
        let width = 120;
        // At the extremes the ReLU gate saturates, so the handwired engine
        // agrees with the standard conditional engine.
        for t in [0.0, 0.3, 0.5, 1.5, 2.0] {
            let std = standard_layout_engine(t, width);
            let (hw, _) = handwired_layout_engine(t, width);
            assert_eq!(std.sidebar_width, hw.sidebar_width);
            assert_eq!(std.main_width, hw.main_width);
        }
    }
}
