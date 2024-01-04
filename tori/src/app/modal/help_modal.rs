use super::{get_modal_chunk, Message, Modal};

use tui::{
    layout::{Alignment, Constraint},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::component::Mode,
    command::Command,
    config::{shortcuts::InputStr, Config},
    error::Result,
    events::Event,
};

/// A modal box that asks for user input
#[derive(Debug, Default)]
pub struct HelpModal {
    playlists_dir: String,
    rows: Vec<Row<'static>>,
}

impl HelpModal {
    pub fn new() -> Self {
        let config = Config::global();

        let playlists_dir = format!("playlists folder: {}", config.playlists_dir);

        let mut entries: Vec<_> = config.keybindings.0.iter().collect();
        entries.sort_unstable_by(|(k0, _), (k1, _)| k0.cmp(k1));
        let max_key_length = entries
            .iter()
            .map(|(k, _)| k.0.width())
            .max()
            .unwrap_or_default();
        let pad = |x: &str| format!("{}{}", " ".repeat(max_key_length - x.width()), x);

        let rows: Vec<_> = entries
            .chunks(3)
            .map(|chunk| {
                let make_cell = |(k, v): &(&InputStr, &Command)| {
                    Line::from(vec![
                        Span::styled(pad(&k.0), Style::default().fg(Color::LightBlue)),
                        Span::raw(format!(" {:?}", v)),
                    ])
                };

                Row::new(chunk.iter().map(make_cell).collect::<Vec<_>>())
            })
            .collect();

        Self {
            playlists_dir,
            rows,
        }
    }
}

impl Modal for HelpModal {
    fn apply_style(&mut self, _style: Style) {}

    fn handle_event(&mut self, event: Event) -> Result<Message> {
        if let Event::Terminal(crossterm::event::Event::Key(_)) = event {
            return Ok(Message::Quit);
        }
        Ok(Message::Nothing)
    }

    fn render(&mut self, frame: &mut Frame) {
        let size = frame.size();
        let mut chunk = get_modal_chunk(size);
        chunk.y = 3;
        chunk.height = frame.size().height.saturating_sub(6);

        let block = Block::default()
            .title(" Help ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightBlue));

        let paragraph = Paragraph::new(self.playlists_dir.as_str())
            .block(block)
            .alignment(Alignment::Center);

        let widths = [
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ];
        let table = Table::default()
            .rows(self.rows.clone())
            .widths(widths)
            .column_spacing(1);

        frame.render_widget(Clear, chunk);
        frame.render_widget(paragraph, chunk);

        chunk.x += 1;
        chunk.y += 3;
        chunk.width -= 2;
        chunk.height -= 3;
        frame.render_widget(table, chunk);
    }

    fn mode(&self) -> Mode {
        Mode::Insert
    }
}
