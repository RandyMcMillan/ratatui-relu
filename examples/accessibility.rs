//! Multi-panel layout with an accessibility mode toggle.
//!
//! Keeps the multi-panel structure and adds a smooth transition to an
//! accessible layout with larger panels and higher contrast.
//!
//! Run in a real terminal with: cargo run --example accessibility

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
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use ratatui_relu::animation::Transition;

struct App {
    header: Transition,
    left_sidebar: Transition,
    right_sidebar: Transition,
    footer: Transition,
    // 0.0 = normal layout, 1.0 = accessibility layout.
    mode: Transition,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            // Start all panels open.
            header: Transition::new(1.0, 4.0),
            left_sidebar: Transition::new(1.0, 4.0),
            right_sidebar: Transition::new(1.0, 4.0),
            footer: Transition::new(1.0, 4.0),
            // Start in normal mode. Use 1.0 here to start in accessibility mode.
            mode: Transition::new(0.0, 2.0),
            should_quit: false,
        }
    }

    fn toggle_accessibility(&mut self) {
        self.mode.toggle_between(0.0, 1.0);
    }

    fn update(&mut self, dt: Duration) {
        self.header.update(dt);
        self.left_sidebar.update(dt);
        self.right_sidebar.update(dt);
        self.footer.update(dt);
        self.mode.update(dt);
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
                KeyCode::Char('a') => app.toggle_accessibility(),
                KeyCode::Char('h') => app.header.toggle_between(0.0, 1.0),
                KeyCode::Char('l') => app.left_sidebar.toggle_between(0.0, 1.0),
                KeyCode::Char('r') => app.right_sidebar.toggle_between(0.0, 1.0),
                KeyCode::Char('f') => app.footer.toggle_between(0.0, 1.0),
                KeyCode::Char(' ') => {
                    app.header.toggle_between(0.0, 1.0);
                    app.left_sidebar.toggle_between(0.0, 1.0);
                    app.right_sidebar.toggle_between(0.0, 1.0);
                    app.footer.toggle_between(0.0, 1.0);
                }
                KeyCode::Up => {
                    app.header.set_target(0.0);
                    app.footer.set_target(1.0);
                }
                KeyCode::Down => {
                    app.footer.set_target(0.0);
                    app.header.set_target(1.0);
                }
                KeyCode::Right => {
                    if app.right_sidebar.current() < 0.5 {
                        app.left_sidebar.set_target(1.0);
                    } else {
                        app.right_sidebar.toggle_between(0.0, 1.0);
                    }
                }
                KeyCode::Left => {
                    if app.left_sidebar.current() < 0.5 {
                        app.right_sidebar.set_target(1.0);
                    } else {
                        app.left_sidebar.toggle_between(0.0, 1.0);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();
    let t = app.mode.current();

    // Accessibility mode scales panel sizes and switches to high contrast.
    let header_height =
        lerp(app.header.current() * 5.0, app.header.current() * 8.0, t).round() as u16;
    let footer_height =
        lerp(app.footer.current() * 5.0, app.footer.current() * 8.0, t).round() as u16;
    let left_width = lerp(
        app.left_sidebar.current() * area.width as f64 * 0.25,
        app.left_sidebar.current() * area.width as f64 * 0.35,
        t,
    )
    .round() as u16;
    let right_width = lerp(
        app.right_sidebar.current() * area.width as f64 * 0.25,
        app.right_sidebar.current() * area.width as f64 * 0.35,
        t,
    )
    .round() as u16;

    let (border_color, bg_color, fg_color) = if t > 0.5 {
        (Color::Yellow, Color::White, Color::Black)
    } else {
        (Color::Blue, Color::Black, Color::White)
    };

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
        .constraints([
            Constraint::Length(left_width),
            Constraint::Min(0),
            Constraint::Length(right_width),
        ])
        .split(vertical[1]);

    let mode_label = if app.mode.target() > 0.5 {
        "Accessibility ON"
    } else {
        "Accessibility OFF"
    };
    let header_text = Text::from(vec![
        Line::from(vec![
            Span::raw("Accessibility Demo | mode: "),
            Span::styled(
                mode_label,
                Style::default()
                    .fg(if app.mode.target() > 0.5 {
                        Color::Green
                    } else {
                        Color::Red
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" | blend: {:.2}", t)),
        ]),
        Line::from("[a] accessibility  [h/l/r/f] toggle  [Space] all  [arrows] move  [q] quit"),
    ]);
    let header =
        Paragraph::new(header_text).block(Block::default().title("Header").borders(Borders::ALL));
    f.render_widget(header, vertical[0]);

    render_panel(f, "Left", horizontal[0], Color::Green, bg_color, fg_color);
    render_panel(f, "Main", horizontal[1], border_color, bg_color, fg_color);
    render_panel(
        f,
        "Right",
        horizontal[2],
        Color::Magenta,
        bg_color,
        fg_color,
    );

    let footer_block = Block::default()
        .title("Footer")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let footer_inner = footer_block.inner(vertical[2]);
    f.render_widget(footer_block, vertical[2]);

    let help = Paragraph::new("Accessibility mode adds larger panels and high-contrast colors.")
        .alignment(Alignment::Center);
    f.render_widget(help, footer_inner);
}

fn render_panel(
    f: &mut Frame,
    title: &str,
    area: Rect,
    border_color: Color,
    bg_color: Color,
    fg_color: Color,
) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(bg_color).fg(fg_color));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width > 0 && inner.height > 0 {
        let text = Paragraph::new(format!("{} area", title))
            .alignment(Alignment::Center)
            .style(Style::default().bg(bg_color).fg(fg_color));
        f.render_widget(text, inner);
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
