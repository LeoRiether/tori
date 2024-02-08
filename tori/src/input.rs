use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::{
    prelude::*,
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug, Default, Clone)]
pub struct Input {
    pub value: String,
    pub cursor: usize,
}

impl Input {
    pub fn handle_event(&mut self, key: KeyEvent) {
        use KeyCode::*;
        match key.code {
            Char(c) => {
                if self.cursor == self.value.len() {
                    self.value.push(c);
                } else {
                    self.value.insert(self.cursor, c);
                }
                self.move_cursor(1);
            }
            Backspace if key.modifiers & KeyModifiers::ALT != KeyModifiers::NONE => {
                // Remove trailing whitespace
                while let Some(c) = self.value.pop() {
                    if !c.is_whitespace() {
                        break;
                    }
                }
                // Remove word
                while let Some(c) = self.value.pop() {
                    if c.is_whitespace() {
                        self.value.push(c);
                        break;
                    }
                }
            }
            Backspace => {
                if self.cursor > 0 {
                    self.move_cursor(-1);
                    self.value.remove(self.cursor);
                }
            }
            Delete => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                }
            }
            Left => {
                self.move_cursor(-1);
            }
            Right => {
                self.move_cursor(1);
            }
            Home => {
                self.cursor = 0;
            }
            End => {
                self.cursor = self.value.len();
            }
            _ => {}
        }
    }

    fn move_cursor(&mut self, x: isize) {
        let inc = |y: usize| (y as isize + x).min(self.value.len() as isize).max(0) as usize;
        self.cursor = inc(self.cursor);

        while !self.value.is_char_boundary(self.cursor) {
            self.cursor = inc(self.cursor);
        }
    }
}

pub struct InputWidget<'a> {
    input: &'a Input,
    block: Option<Block<'a>>,
    style: Style,
}

impl<'a> InputWidget<'a> {
    pub fn new(input: &Input) -> InputWidget<'_> {
        InputWidget {
            input,
            block: None,
            style: Style::default(),
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Widget for InputWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let input = self.input;
        let (left, right) = input.value.split_at(input.cursor);

        let (cursor, right) = right
            .char_indices()
            .next()
            .map(|(i, _)| right.split_at(i))
            .unwrap_or(("", right));

        let mut paragraph = Paragraph::new(Line::from(vec![
            Span::styled(left, self.style),
            Span::styled(cursor, Style::default().add_modifier(Modifier::REVERSED)),
            right.into(),
        ]));

        if let Some(block) = self.block {
            paragraph = paragraph.block(block);
        }

        paragraph.render(area, buf);
    }
}
