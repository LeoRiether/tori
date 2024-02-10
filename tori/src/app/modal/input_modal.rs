use super::{get_modal_chunk, Modal};

use crossterm::event::{Event, KeyCode};
use std::sync::Mutex;
use std::{borrow::Cow, mem};
use tui::{
    layout::Alignment,
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use crate::events::channel::Tx;
use crate::{error::Result, events::Action, input::Input};

/// A modal box that asks for user input
#[derive(Debug, Default)]
pub struct InputModal<F> {
    title: Cow<'static, str>,
    input: Input,
    scroll: Mutex<u16>,
    style: Style,
    on_commit: Option<F>,
}

impl<F> InputModal<F>
where
    F: FnOnce(String) -> Action + Send + Sync,
{
    pub fn new(title: impl Into<Cow<'static, str>>) -> Self {
        Self {
            title: title.into(),
            scroll: Mutex::new(0),
            input: Input::default(),
            style: Style::default().fg(Color::LightBlue),
            on_commit: None,
        }
    }

    pub fn set_input(mut self, input: String) -> Self {
        self.input.value = input;
        self.input.cursor = self.input.value.len();
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn on_commit(mut self, action: F) -> Self {
        self.on_commit = Some(action);
        self
    }
}

impl<F> Modal for InputModal<F>
where
    F: FnOnce(String) -> Action + Send + Sync,
{
    fn handle_event(&mut self, tx: Tx, event: Event) -> Result<Option<Action>> {
        let key = match event {
            Event::Key(key) => key,
            _ => return Ok(None),
        };

        Ok(match key.code {
            KeyCode::Esc => Some(Action::CloseModal),
            KeyCode::Enter => {
                let input = mem::take(&mut self.input).value;
                tx.send(Action::CloseModal);
                self.on_commit.take().map(|f| f(input))
            }
            _ => {
                self.input.handle_event(key);
                None
            }
        })
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let chunk = get_modal_chunk(area);
        let prefix = " ❯ ";
        let scroll = self.calculate_scroll(chunk.width - prefix.len() as u16);

        let block = Block::default()
            .title(self.title.as_ref())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(self.style);

        // split input as [left, cursor, right]
        let (left, right) = self.input.value.split_at(self.input.cursor);
        let mut indices = right.char_indices();
        let (in_cursor, right) = indices
            .next()
            .map(|_| right.split_at(indices.next().map(|(w, _)| w).unwrap_or(right.len())))
            .unwrap_or((" ", ""));

        let paragraph = Paragraph::new(vec![
            Line::from(vec![]), // empty first line
            Line::from(vec![
                Span::styled(prefix, self.style),
                Span::raw(left),
                Span::styled(in_cursor, Style::default().add_modifier(Modifier::REVERSED)),
                Span::raw(right),
            ]),
        ])
        .block(block)
        .scroll((0, scroll))
        .alignment(Alignment::Left);

        Clear.render(chunk, buf);
        paragraph.render(chunk, buf);
    }
}

impl<F> InputModal<F> {
    /// Updates and calculates the Paragraph's scroll based on the current cursor and input
    fn calculate_scroll(&self, chunk_width: u16) -> u16 {
        let mut scroll = self.scroll.lock().unwrap();
        if self.input.cursor as u16 > *scroll + chunk_width - 1 {
            *scroll = self.input.cursor as u16 + 1 - chunk_width;
        }

        if (self.input.cursor as u16) <= *scroll {
            *scroll = (self.input.cursor as u16).saturating_sub(1);
        }

        *scroll
    }
}
