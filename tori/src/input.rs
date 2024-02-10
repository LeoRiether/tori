use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::{
    prelude::*,
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug, Default, Clone, Copy)]
pub enum InputResponse {
    #[default]
    NotHandled,
    Handled,
}

#[derive(Debug, Default, Clone)]
pub struct Input {
    pub value: String,
    pub cursor: usize,
}

impl Input {
    pub fn new(value: impl Into<String>) -> Input {
        Input {
            value: value.into(),
            cursor: 0,
        }
    }

    pub fn handle_event(&mut self, key: KeyEvent) -> InputResponse {
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
            _ => return InputResponse::NotHandled,
        }
        InputResponse::Handled
    }

    fn move_cursor(&mut self, x: isize) {
        let inc = |y: usize| (y as isize + x).min(self.value.len() as isize).max(0) as usize;
        self.cursor = inc(self.cursor);

        while !self.value.is_char_boundary(self.cursor) {
            self.cursor = inc(self.cursor);
        }
    }

    pub fn split_at_cursor(&self) -> (&str, &str, &str) {
        let (left, right) = self.value.split_at(self.cursor);

        let (cursor, right) = right
            .char_indices()
            .next()
            .map(|(i, _)| right.split_at(i))
            .unwrap_or(("", right));

        (left, cursor, right)
    }

    pub fn styled(&self, style: Style) -> Line {
        let (left, cursor, right) = self.split_at_cursor();
        Line::from(vec![
            Span::styled(left, style),
            Span::styled(cursor, style.reversed().underlined()),
            Span::styled(right, style),
        ])
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
        let mut paragraph = Paragraph::new(self.input.styled(self.style));
        if let Some(block) = self.block {
            paragraph = paragraph.block(block);
        }

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_cursor_ascii() {
        let mut input = Input::new("input cursor");
        input.value = "Hello World!".into();
        assert_eq!(input.cursor, 0);

        input.move_cursor(1);
        assert_eq!(input.cursor, 1);

        input.move_cursor(1);
        assert_eq!(input.cursor, 2);

        input.move_cursor(-1);
        assert_eq!(input.cursor, 1);

        input.move_cursor(-1);
        assert_eq!(input.cursor, 0);

        input.move_cursor(-1);
        assert_eq!(input.cursor, 0);

        input.move_cursor(1000);
        assert_eq!(input.cursor, input.value.len());
    }

    #[test]
    fn test_input_cursor_unicode() {
        let mut input = Input::new("input cursor");
        input.value = "おはよう".into();
        assert_eq!(input.cursor, 0);

        input.move_cursor(1);
        assert_eq!(input.cursor, 3);

        input.move_cursor(1);
        assert_eq!(input.cursor, 6);

        input.move_cursor(-1);
        assert_eq!(input.cursor, 3);

        input.move_cursor(-1);
        assert_eq!(input.cursor, 0);
    }
}
