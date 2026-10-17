//! Clickable, interactive virtual keyboard widget for Ratatui.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::mouse::is_mouse_over;

/// Rows of keys in the keyboard graphic.
const KEY_ROWS: &[&[char]] = &[
    &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '='],
    &['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '[', ']', '\\'],
    &['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ';', '\''],
    &['Z', 'X', 'C', 'V', 'B', 'N', 'M', ',', '.', '/'],
    &[' '],
];

/// Actions produced by the keyboard widget interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardAction {
    Char(char),
    Space,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Selected,
    Active,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonTheme {
    pub text: Color,
    pub background: Color,
    pub highlight: Color,
    pub shadow: Color,
}

impl ButtonTheme {
    pub const BLUE: Self = Self {
        text: Color::Rgb(16, 24, 48),
        background: Color::Rgb(48, 72, 144),
        highlight: Color::Rgb(64, 96, 192),
        shadow: Color::Rgb(32, 48, 96),
    };
}

#[derive(Debug, Clone)]
pub struct Button<'a> {
    label: Line<'a>,
    theme: ButtonTheme,
    state: ButtonState,
}

impl<'a> Button<'a> {
    pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
        Self {
            label: label.into(),
            theme: ButtonTheme::BLUE,
            state: ButtonState::Normal,
        }
    }

    pub const fn theme(mut self, theme: ButtonTheme) -> Self {
        self.theme = theme;
        self
    }

    pub const fn state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }

    const fn colors(&self) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        match self.state {
            ButtonState::Normal => (theme.background, theme.text, theme.shadow, theme.highlight),
            ButtonState::Selected => (theme.highlight, theme.text, theme.shadow, theme.highlight),
            ButtonState::Active => (theme.background, theme.text, theme.highlight, theme.shadow),
        }
    }
}

impl Widget for Button<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::{Block, BorderType, Borders};
        let (background, text, _shadow, highlight) = self.colors();
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(highlight).bg(background));
        let inner = block.inner(area);
        buf.set_style(area, Style::new().bg(background).fg(text));
        ratatui::widgets::Widget::render(block, area, buf);
        if inner.width > 0 && inner.height > 0 {
            buf.set_line(
                inner.x + (inner.width.saturating_sub(self.label.width() as u16)) / 2,
                inner.y + (inner.height.saturating_sub(1)) / 2,
                &self.label,
                inner.width,
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyboardState {
    pub button_states: Vec<ButtonState>,
    pub last_areas: Vec<Option<Rect>>,
    pub keys: Vec<char>,
    pub last_output: String,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardState {
    pub fn new() -> Self {
        let mut keys = Vec::new();
        for row in KEY_ROWS {
            keys.extend_from_slice(row);
        }
        Self {
            button_states: vec![ButtonState::default(); keys.len()],
            last_areas: vec![None; keys.len()],
            keys,
            last_output: String::new(),
        }
    }

    pub fn handle_mouse(&mut self, event: MouseEvent) -> Option<KeyboardAction> {
        let mut clicked: Option<usize> = None;

        for (i, area) in self.last_areas.iter().enumerate() {
            if let Some(area) = area {
                if is_mouse_over(*area, event.column, event.row)
                    && matches!(event.kind, MouseEventKind::Down(MouseButton::Left))
                {
                    clicked = Some(i);
                }
            }
        }

        // update hover states
        for (i, bs) in self.button_states.iter_mut().enumerate() {
            if let Some(area) = self.last_areas[i] {
                let over = is_mouse_over(area, event.column, event.row);
                match event.kind {
                    MouseEventKind::Moved => {
                        if over && *bs != ButtonState::Active {
                            *bs = ButtonState::Selected;
                        } else if !over && *bs == ButtonState::Selected {
                            *bs = ButtonState::Normal;
                        }
                    }
                    MouseEventKind::Down(MouseButton::Left) if over => {
                        *bs = if *bs == ButtonState::Active {
                            ButtonState::Normal
                        } else {
                            ButtonState::Active
                        };
                    }
                    _ => {}
                }
            }
        }

        if let Some(i) = clicked {
            if let Some(&key) = self.keys.get(i) {
                self.last_output = if key == ' ' {
                    "Clicked: [Space]".to_string()
                } else {
                    format!("Clicked: {}", key)
                };
                return if key == ' ' {
                    Some(KeyboardAction::Space)
                } else {
                    Some(KeyboardAction::Char(key))
                };
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct KeyboardWidget {
    pub rows: Vec<Vec<char>>,
}

impl Default for KeyboardWidget {
    fn default() -> Self {
        Self {
            rows: KEY_ROWS.iter().map(|r| r.to_vec()).collect(),
        }
    }
}

impl StatefulWidget for KeyboardWidget {
    type State = KeyboardState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut y = area.y;
        let mut idx = 0usize;

        for row in &self.rows {
            let indent = match row.len() {
                13 => 0u16,
                12 => 1u16,
                11 => 2u16,
                10 => 3u16,
                1 => 18u16,
                _ => 0u16,
            };
            let mut x = area.x + indent;

            for key in row {
                let label = if *key == ' ' { "Space".to_string() } else { key.to_string() };
                let w: u16 = 5;
                let h: u16 = 3;
                let btn_area = Rect::new(x, y, w, h);

                if x + w <= area.x + area.width && y + h <= area.y + area.height {
                    let st = state.button_states.get(idx).copied().unwrap_or(ButtonState::Normal);
                    let button = Button::new(label).state(st);
                    if let Some(slot) = state.last_areas.get_mut(idx) {
                        *slot = Some(btn_area);
                    }
                    button.render(btn_area, buf);
                }
                idx += 1;
                x += w + 1;
            }
            y += 4;
            if y >= area.y + area.height {
                break;
            }
        }
    }
}