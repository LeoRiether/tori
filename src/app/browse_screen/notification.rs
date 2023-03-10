use std::time::{Duration, Instant};

use tui::{
    layout::Rect,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap, Clear},
    Frame, style::{Color, Style},
};

use crate::app::MyBackend;

#[derive(Debug)]
pub struct Notification {
    pub text: String,
    pub show_until: Instant,
    pub color: Color,
}

impl Default for Notification {
    fn default() -> Self {
        Self {
            text: String::new(),
            show_until: Instant::now(),
            color: Color::White,
        }
    }
}

impl Notification {
    pub fn new(text: String, duration: Duration) -> Self {
        Self {
            text,
            show_until: Instant::now() + duration,
            ..Default::default()
        }
    }

    pub fn colored(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.show_until
    }

    pub fn render(&self, frame: &mut Frame<'_, MyBackend>) {
        if self.is_expired() {
            return;
        }

        let size = frame.size();
        let chunk = Rect {
            x: size.width - 40,
            y: size.height - 5,
            width: 39,
            height: 4,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.color));

        let text = Paragraph::new(self.text.as_ref())
            .block(block)
            .style(Style::default().fg(self.color))
            .wrap(Wrap{ trim: true });

        frame.render_widget(Clear, chunk);
        frame.render_widget(text, chunk);
    }
}
