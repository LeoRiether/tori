use crate::app::MyBackend;

use crossterm::event::{Event, KeyCode};
use std::{borrow::Cow, error::Error, path::Path};
use tui::{
    layout,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};

#[derive(Debug, Default)]
pub struct PlaylistsPane {
    pub playlists: Vec<String>,
    selected: usize,
}

impl PlaylistsPane {
    pub fn from_dir<P: AsRef<Path>>(path: P) -> Self {
        let mut me = Self::default();
        me.reload_from_dir(path);
        me
    }

    pub fn selected_item(&self) -> &str {
        &self.playlists[self.selected] 
    }

    pub fn reload_from_dir<P: AsRef<Path>>(&mut self, path: P) {
        let dir = std::fs::read_dir(path).unwrap();

        use std::fs::DirEntry;
        use std::io::Error;
        let extract_playlist_name = |entry: Result<DirEntry, Error>| {
            entry
                .unwrap()
                .file_name()
                .into_string()
                .unwrap()
                .trim_end_matches(".m3u")
                .to_string()
        };

        self.playlists = dir
            .into_iter()
            .map(extract_playlist_name)
            .collect();
    }

    pub fn render(
        &mut self,
        is_focused: bool,
        frame: &mut Frame<'_, MyBackend>,
        chunk: layout::Rect,
    ) {
        let mut block = Block::default()
            .title(" playlists ")
            .borders(Borders::LEFT | Borders::BOTTOM | Borders::TOP)
            .border_type(BorderType::Plain);

        if is_focused {
            block = block.border_style(Style::default().fg(Color::LightBlue));
        }

        let mut playlists: Vec<_> = self
            .playlists
            .iter()
            .map(|s| ListItem::new(Cow::from(s)))
            .collect();

        // Highlight selected playlists
        playlists[self.selected] =
            std::mem::replace(&mut playlists[self.selected], ListItem::new(""))
                .style(Style::default().bg(Color::Blue).fg(Color::White));

        let widget = List::new(playlists).block(block);
        frame.render_widget(widget, chunk);
    }

    pub fn handle_event(&mut self, event: Event) -> Result<(), Box<dyn Error>> {
        match event {
            Event::Key(event) => match event.code {
                KeyCode::Up => self.select_prev(),
                KeyCode::Down => self.select_next(),
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    pub fn select_next(&mut self) {
        self.selected = wrap_inc(self.selected, self.playlists.len());
    }

    pub fn select_prev(&mut self) {
        self.selected = wrap_dec(self.selected, self.playlists.len());
    }
}

fn wrap_inc(x: usize, modulo: usize) -> usize {
    if x == modulo - 1 {
        0
    } else {
        x + 1
    }
}

fn wrap_dec(x: usize, modulo: usize) -> usize {
    if x == 0 {
        modulo - 1
    } else {
        x - 1
    }
}
