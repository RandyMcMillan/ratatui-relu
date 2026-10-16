//! A Ratatui example that demonstrates different types of flex layouts.
//!
//! Ported from the Ratatui `flex` example.
//!
//! Run with: cargo run --example flex

use std::io::{self, Stdout, stdout};
use std::num::NonZeroUsize;
use std::sync::LazyLock;
use std::time::Instant;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::Constraint::{self, Fill, Length, Max, Min, Percentage, Ratio},
    layout::{Alignment, Flex, Layout, Rect},
    style::palette::tailwind,
    style::{Color, Modifier, Style, Stylize},
    symbols::{self, line},
    text::{Line, Text},
    widgets::{
        Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Tabs,
        Widget,
    },
};
use ratatui_relu::animation::Transition;
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

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
    let result = App::new().run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

static THEME: LazyLock<Theme> = LazyLock::new(Theme::new);

const EXAMPLE_DATA: &[(&str, &[Constraint])] = &[
    (
        "Min(u16) takes any excess space always",
        &[Length(10), Min(10), Max(10), Percentage(10), Ratio(1, 10)],
    ),
    (
        "Fill(u16) takes any excess space always",
        &[Length(20), Percentage(20), Ratio(1, 5), Fill(1)],
    ),
    (
        "Here's all constraints in one line",
        &[
            Length(10),
            Min(10),
            Max(10),
            Percentage(10),
            Ratio(1, 10),
            Fill(1),
        ],
    ),
    ("", &[Max(50), Min(50)]),
    ("", &[Max(20), Length(10)]),
    ("", &[Max(20), Length(10)]),
    (
        "Min grows always but also allows Fill to grow",
        &[Percentage(50), Fill(1), Fill(2), Min(50)],
    ),
    (
        "In `Legacy`, the last constraint of lowest priority takes excess space",
        &[Length(20), Length(20), Percentage(20)],
    ),
    ("", &[Length(20), Percentage(20), Length(20)]),
    (
        "A lowest priority constraint will be broken before a high priority constraint",
        &[Ratio(1, 4), Percentage(20)],
    ),
    (
        "`Length` is higher priority than `Percentage`",
        &[Percentage(20), Length(10)],
    ),
    (
        "`Min/Max` is higher priority than `Length`",
        &[Length(10), Max(20)],
    ),
    ("", &[Length(100), Min(20)]),
    (
        "`Length` is higher priority than `Min/Max`",
        &[Max(20), Length(10)],
    ),
    ("", &[Min(20), Length(90)]),
    (
        "Fill is the lowest priority and will fill any excess space",
        &[Fill(1), Ratio(1, 4)],
    ),
    (
        "Fill can be used to scale proportionally with other Fill blocks",
        &[Fill(1), Percentage(20), Fill(2)],
    ),
    ("", &[Ratio(1, 3), Percentage(20), Ratio(2, 3)]),
    (
        "Legacy will stretch the last lowest priority constraint\nStretch will only stretch equal weighted constraints",
        &[Length(20), Length(15)],
    ),
    ("", &[Percentage(20), Length(15)]),
    (
        "`Fill(u16)` fills up excess space, but is lower priority to spacers.\ni.e. Fill will only have widths in Flex::Stretch and Flex::Legacy",
        &[Fill(1), Fill(1)],
    ),
    ("", &[Length(20), Length(20)]),
    (
        "When not using `Flex::Stretch` or `Flex::Legacy`,\n`Min(u16)` and `Max(u16)` collapse to their lowest values",
        &[Min(20), Max(20)],
    ),
    ("", &[Max(20)]),
    ("", &[Min(20), Max(20), Length(20), Length(20)]),
    ("", &[Fill(0), Fill(0)]),
    (
        "`Fill(1)` can be to scale with respect to other `Fill(2)`",
        &[Fill(1), Fill(2)],
    ),
    ("", &[Fill(1), Min(10), Max(10), Fill(2)]),
    (
        "`Fill(0)` collapses if there are other non-zero `Fill(_)`\nconstraints. e.g. `[Fill(0), Fill(0), Fill(1)]`:",
        &[Fill(0), Fill(0), Fill(1)],
    ),
];

const ANIMATION_SPEED: f64 = 40.0;

#[derive(Clone, Copy)]
struct App {
    selected_tab: SelectedTab,
    scroll_offset: u16,
    spacing: Transition,
    state: AppState,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Example {
    constraints: Vec<Constraint>,
    description: String,
    flex: Flex,
    spacing: u16,
}

/// Tabs for the different layouts.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, FromRepr, Display, EnumIter)]
enum SelectedTab {
    #[default]
    Legacy,
    Start,
    Center,
    End,
    SpaceAround,
    SpaceEvenly,
    SpaceBetween,
}

impl Default for App {
    fn default() -> Self {
        Self {
            selected_tab: SelectedTab::default(),
            scroll_offset: 0,
            spacing: Transition::new(0.0, ANIMATION_SPEED),
            state: AppState::default(),
        }
    }
}

impl App {
    fn new() -> Self {
        Self::default()
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        let cache_size = EXAMPLE_DATA.len() * SelectedTab::iter().len() * 100;
        Layout::init_cache(NonZeroUsize::new(cache_size).unwrap());

        let mut last_tick = Instant::now();
        while self.is_running() {
            let now = Instant::now();
            self.spacing.update(now - last_tick);
            last_tick = now;

            terminal.draw(|frame| frame.render_widget(*self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn is_running(self) -> bool {
        self.state == AppState::Running
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                KeyCode::Char('l') | KeyCode::Right => self.next(),
                KeyCode::Char('h') | KeyCode::Left => self.previous(),
                KeyCode::Char('j') | KeyCode::Down => self.down(),
                KeyCode::Char('k') | KeyCode::Up => self.up(),
                KeyCode::Char('g') | KeyCode::Home => self.top(),
                KeyCode::Char('G') | KeyCode::End => self.bottom(),
                KeyCode::Char('+') => self.increment_spacing(),
                KeyCode::Char('-') => self.decrement_spacing(),
                _ => (),
            }
        }
        Ok(())
    }

    fn next(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    fn previous(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    const fn up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    fn down(&mut self) {
        self.scroll_offset = self
            .scroll_offset
            .saturating_add(1)
            .min(max_scroll_offset());
    }

    const fn top(&mut self) {
        self.scroll_offset = 0;
    }

    fn bottom(&mut self) {
        self.scroll_offset = max_scroll_offset();
    }

    fn increment_spacing(&mut self) {
        self.spacing.set_target(self.spacing.target() + 1.0);
    }

    fn decrement_spacing(&mut self) {
        self.spacing.set_target(self.spacing.target() - 1.0);
    }

    const fn quit(&mut self) {
        self.state = AppState::Quit;
    }
}

fn max_scroll_offset() -> u16 {
    example_height()
        - EXAMPLE_DATA
            .last()
            .map_or(0, |(desc, _)| get_description_height(desc) + 4)
}

fn example_height() -> u16 {
    EXAMPLE_DATA
        .iter()
        .map(|(desc, _)| get_description_height(desc) + 4)
        .sum()
}

impl Widget for App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Length(3), Length(1), Fill(0)]);
        let chunks = split::<3>(area, layout);
        let tabs = chunks[0];
        let axis = chunks[1];
        let demo = chunks[2];

        self.tabs().render(tabs, buf);
        let scroll_needed = self.render_demo(demo, buf);
        let axis_width = if scroll_needed {
            axis.width.saturating_sub(1)
        } else {
            axis.width
        };
        Self::axis(axis_width, self.spacing.current() as u16).render(axis, buf);
    }
}

impl App {
    fn tabs(self) -> impl Widget {
        let tab_titles = SelectedTab::iter().map(|tab| SelectedTab::to_tab_title(tab, *THEME));
        let block = Block::new()
            .title("Flex Layouts ".bold())
            .title(" Use ◄ ► to change tab, ▲ ▼  to scroll, - + to change spacing ");
        Tabs::new(tab_titles)
            .block(block)
            .highlight_style(Modifier::REVERSED)
            .select(self.selected_tab as usize)
            .divider(" ")
            .padding("", "")
    }

    fn axis(width: u16, spacing: u16) -> impl Widget {
        let width = width as usize;
        let label = if spacing != 0 {
            format!("{width} px (gap: {spacing} px)")
        } else {
            format!("{width} px")
        };
        let bar_width = width.saturating_sub(2);
        let width_bar = format!("<{label:-^bar_width$}>");
        Paragraph::new(width_bar.dark_gray()).centered()
    }

    fn render_demo(self, area: Rect, buf: &mut Buffer) -> bool {
        let height = example_height();
        let demo_area = Rect::new(0, 0, area.width, height);
        let mut demo_buf = Buffer::empty(demo_area);

        let scrollbar_needed = self.scroll_offset != 0 || height > area.height;
        let content_area = if scrollbar_needed {
            Rect {
                width: demo_area.width - 1,
                ..demo_area
            }
        } else {
            demo_area
        };

        let mut spacing = self.spacing.current() as u16;
        self.selected_tab
            .render(content_area, &mut demo_buf, &mut spacing);

        let visible_content = demo_buf
            .content
            .into_iter()
            .skip((area.width * self.scroll_offset) as usize)
            .take(area.area() as usize);
        for (i, cell) in visible_content.enumerate() {
            let x = i as u16 % area.width;
            let y = i as u16 / area.width;
            buf[(area.x + x, area.y + y)] = cell;
        }

        if scrollbar_needed {
            let area = area.intersection(buf.area);
            let mut state = ScrollbarState::new(max_scroll_offset() as usize)
                .position(self.scroll_offset as usize);
            Scrollbar::new(ScrollbarOrientation::VerticalRight).render(area, buf, &mut state);
        }
        scrollbar_needed
    }
}

impl SelectedTab {
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    fn to_tab_title(value: Self, theme: Theme) -> Line<'static> {
        let text = value.to_string();
        let color = match value {
            Self::Legacy => theme.legacy_tab,
            Self::Start => theme.start_tab,
            Self::Center => theme.center_tab,
            Self::End => theme.end_tab,
            Self::SpaceEvenly => theme.space_evenly_tab,
            Self::SpaceBetween => theme.space_between_tab,
            Self::SpaceAround => theme.space_around_tab,
        };
        format!(" {text} ").fg(color).bg(Color::Black).into()
    }
}

impl StatefulWidget for SelectedTab {
    type State = u16;
    fn render(self, area: Rect, buf: &mut Buffer, spacing: &mut Self::State) {
        let spacing = *spacing;
        match self {
            Self::Legacy => Self::render_examples(area, buf, Flex::Legacy, spacing),
            Self::Start => Self::render_examples(area, buf, Flex::Start, spacing),
            Self::Center => Self::render_examples(area, buf, Flex::Center, spacing),
            Self::End => Self::render_examples(area, buf, Flex::End, spacing),
            Self::SpaceEvenly => Self::render_examples(area, buf, Flex::SpaceEvenly, spacing),
            Self::SpaceBetween => Self::render_examples(area, buf, Flex::SpaceBetween, spacing),
            Self::SpaceAround => Self::render_examples(area, buf, Flex::SpaceAround, spacing),
        }
    }
}

impl SelectedTab {
    fn render_examples(area: Rect, buf: &mut Buffer, flex: Flex, spacing: u16) {
        let heights = EXAMPLE_DATA
            .iter()
            .map(|(desc, _)| get_description_height(desc) + 4);
        let areas = Layout::vertical(heights).flex(Flex::Start).split(area);
        for (area, (description, constraints)) in areas.iter().zip(EXAMPLE_DATA.iter()) {
            Example::new(constraints, description, flex, spacing).render(*area, buf);
        }
    }
}

impl Example {
    fn new(constraints: &[Constraint], description: &str, flex: Flex, spacing: u16) -> Self {
        Self {
            constraints: constraints.into(),
            description: description.into(),
            flex,
            spacing,
        }
    }
}

impl Widget for Example {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title_height = get_description_height(&self.description);
        let layout = Layout::vertical([Length(title_height), Fill(0)]);
        let chunks = split::<2>(area, layout);
        let title = chunks[0];
        let illustrations = chunks[1];

        let (blocks, spacers) = Layout::horizontal(&self.constraints)
            .flex(self.flex)
            .spacing(self.spacing)
            .split_with_spacers(illustrations);

        if !self.description.is_empty() {
            Paragraph::new(
                self.description
                    .split('\n')
                    .map(|s| format!("// {s}").italic().fg(THEME.description_fg))
                    .map(Line::from)
                    .collect::<Vec<Line>>(),
            )
            .render(title, buf);
        }

        for (block, constraint) in blocks.iter().zip(&self.constraints) {
            Self::illustration(*constraint, block.width, *THEME).render(*block, buf);
        }

        for spacer in spacers.iter() {
            Self::render_spacer(*spacer, buf);
        }
    }
}

impl Example {
    fn render_spacer(spacer: Rect, buf: &mut Buffer) {
        if spacer.width > 1 {
            let corners_only = symbols::border::Set {
                top_left: line::NORMAL.top_left,
                top_right: line::NORMAL.top_right,
                bottom_left: line::NORMAL.bottom_left,
                bottom_right: line::NORMAL.bottom_right,
                vertical_left: " ",
                vertical_right: " ",
                horizontal_top: " ",
                horizontal_bottom: " ",
            };
            Block::bordered()
                .border_set(corners_only)
                .border_style(Style::reset().dark_gray())
                .render(spacer, buf);
        } else {
            Paragraph::new(Text::from(vec![
                Line::from(""),
                Line::from("│"),
                Line::from("│"),
                Line::from(""),
            ]))
            .style(Style::reset().dark_gray())
            .render(spacer, buf);
        }
        let width = spacer.width;
        let label = if width > 4 {
            format!("{width} px")
        } else if width > 2 {
            format!("{width}")
        } else {
            String::new()
        };
        let text = Text::from(vec![
            Line::raw(""),
            Line::raw(""),
            Line::styled(label, Style::reset().dark_gray()),
        ]);
        Paragraph::new(text)
            .style(Style::reset().dark_gray())
            .alignment(Alignment::Center)
            .render(spacer, buf);
    }

    fn illustration(constraint: Constraint, width: u16, theme: Theme) -> impl Widget {
        let main_color = color_for_constraint(constraint, theme);
        let fg_color = Color::White;
        let title = format!("{constraint}");
        let content = format!("{width} px");
        let text = format!("{title}\n{content}");
        let block = Block::bordered()
            .border_set(symbols::border::QUADRANT_OUTSIDE)
            .border_style(Style::reset().fg(main_color).reversed())
            .style(Style::default().fg(fg_color).bg(main_color));
        Paragraph::new(text).centered().block(block)
    }
}

const fn color_for_constraint(constraint: Constraint, theme: Theme) -> Color {
    match constraint {
        Constraint::Min(_) => theme.min_bg,
        Constraint::Max(_) => theme.max_bg,
        Constraint::Length(_) => theme.length_bg,
        Constraint::Percentage(_) => theme.percentage_bg,
        Constraint::Ratio(_, _) => theme.ratio_bg,
        Constraint::Fill(_) => theme.fill_bg,
    }
}

fn get_description_height(s: &str) -> u16 {
    if s.is_empty() {
        0
    } else {
        s.split('\n').count() as u16
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Theme {
    min_bg: Color,
    max_bg: Color,
    length_bg: Color,
    percentage_bg: Color,
    ratio_bg: Color,
    fill_bg: Color,
    legacy_tab: Color,
    start_tab: Color,
    center_tab: Color,
    end_tab: Color,
    space_evenly_tab: Color,
    space_between_tab: Color,
    space_around_tab: Color,
    description_fg: Color,
}

impl Theme {
    pub fn new() -> Self {
        use tailwind::{BLUE, INDIGO, ORANGE, SKY, SLATE};

        let is_true_color = Self::is_true_color_supported();
        let color = |true_color, ansi_color| {
            if is_true_color {
                true_color
            } else {
                ansi_color
            }
        };

        Self {
            min_bg: color(BLUE.c900, Color::Indexed(24)),
            max_bg: color(BLUE.c800, Color::Indexed(25)),
            length_bg: color(SLATE.c700, Color::Indexed(67)),
            percentage_bg: color(SLATE.c800, Color::Indexed(18)),
            ratio_bg: color(SLATE.c900, Color::Indexed(17)),
            fill_bg: color(SLATE.c950, Color::Indexed(16)),
            legacy_tab: color(ORANGE.c400, Color::Indexed(173)),
            start_tab: color(SKY.c400, Color::Indexed(74)),
            center_tab: color(SKY.c300, Color::Indexed(116)),
            end_tab: color(SKY.c200, Color::Indexed(152)),
            space_evenly_tab: color(INDIGO.c400, Color::Indexed(104)),
            space_between_tab: color(INDIGO.c300, Color::Indexed(146)),
            space_around_tab: color(INDIGO.c500, Color::Indexed(68)),
            description_fg: color(SLATE.c400, Color::Indexed(109)),
        }
    }

    fn is_true_color_supported() -> bool {
        let term = std::env::var("TERM_PROGRAM").unwrap_or_default();
        if term == "Apple_Terminal" {
            let term_v = std::env::var("TERM_PROGRAM_VERSION")
                .unwrap_or_default()
                .parse()
                .unwrap_or(0);
            if term_v < 465 {
                return false;
            }
        }
        true
    }
}

fn split<const N: usize>(area: Rect, layout: Layout) -> [Rect; N] {
    layout
        .split(area)
        .as_ref()
        .try_into()
        .unwrap_or([Rect::default(); N])
}
