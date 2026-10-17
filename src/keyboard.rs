//! Clickable keyboard widget.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{StatefulWidget, Widget},
};

use crate::mouse::{Button, ButtonState, is_mouse_over};
use crate::multi_panel::MultiPanelApp;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

/// Rows of keys in the keyboard graphic.
const KEY_ROWS: &[&[char]] = &[
    &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '='],
    &['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '[', ']', '\\'],
    &['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ';', '\''],
    &['Z', 'X', 'C', 'V', 'B', 'N', 'M', ',', '.', '/'],
    &[' '],
];

#[derive(Debug, Default, Clone)]
pub struct KeyboardState {
    pub button_states: Vec<ButtonState>,
    pub keys: Vec<char>,
}

impl KeyboardState {
    pub fn new() -> Self {
        let mut keys = Vec::new();
        for row in KEY_ROWS {
            keys.extend_from_slice(row);
        }
        Self {
            button_states: vec![ButtonState::default(); keys.len()],
            keys,
        }
    }

    pub fn handle_mouse(&mut self, event: MouseEvent, app: &mut MultiPanelApp) -> bool {
        use crossterm::event::KeyCode;
        let mut consumed = false;
        let mut clicked: Option<usize> = None;
        for (i, btn) in self.button_states.iter_mut().enumerate() {
            if btn.handle_mouse(event) {
                consumed = true;
            }
            if matches!(event.kind, MouseEventKind::Down(MouseButton::Left)) {
                if let Some(area) = btn.last_area {
                    if is_mouse_over(area, event.column, event.row) {
                        clicked = Some(i);
                    }
                }
            }
        }
        if let Some(i) = clicked {
            if let Some(key) = self.keys.get(i) {
                app.handle_key(KeyCode::Char(*key));
                consumed = true;
            }
        }
        consumed
    }
}

#[derive(Debug, Default, Clone)]
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

    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, state: &mut Self::State) {
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
                let w: u16 = if *key == ' ' { 9 } else { 3 };
                let h: u16 = 3;
                let btn_area = ratatui::layout::Rect::new(x, y, w, h);
                if x + w <= area.x + area.width && y + h <= area.y + area.height {
                    let button = crate::mouse::Button::new(label);
                    if let Some(bs) = state.button_states.get_mut(idx) {
                        bs.last_area = Some(btn_area);
                        button.render(btn_area, buf, bs);
                    }
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
