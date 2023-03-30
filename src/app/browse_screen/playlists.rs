use crate::app::{filtered_list::FilteredList, App, Mode, MyBackend};
use crate::config::Config;
use crate::events::Event;

use crossterm::event::{KeyCode, KeyEvent, MouseEventKind};
use std::error::Error;
use std::path::PathBuf;
use tui::{
    layout,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};

#[derive(Debug, Default)]
pub struct PlaylistsPane {
    playlists: Vec<String>,
    shown: FilteredList<ListState>,
    filter: String,
}

impl PlaylistsPane {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut me = Self::default();
        me.reload_from_dir()?;
        Ok(me)
    }

    pub fn reload_from_dir(&mut self) -> Result<(), Box<dyn Error>> {
        let dir = std::fs::read_dir(&Config::global().playlists_dir)
            .map_err(|e| format!("Failed to read playlists directory: {}", e))?;

        use std::fs::DirEntry;
        use std::io::Error;
        let extract_playlist_name = |entry: Result<DirEntry, Error>| {
            entry
                .unwrap()
                .file_name()
                .into_string()
                .unwrap()
                .trim_end_matches(".m3u8")
                .to_string()
        };

        self.playlists = dir.into_iter().map(extract_playlist_name).collect();
        self.playlists.sort();
        self.refresh_shown();
        Ok(())
    }

    fn refresh_shown(&mut self) {
        self.shown.filter(&self.playlists, |s| {
            self.filter.is_empty() || s.to_lowercase().contains(&self.filter[1..].to_lowercase())
        });
    }

    pub fn render(
        &mut self,
        is_focused: bool,
        frame: &mut Frame<'_, MyBackend>,
        chunk: layout::Rect,
    ) {
        let title = if !self.filter.is_empty() {
            format!(" {} ", self.filter)
        } else {
            " playlists ".into()
        };

        let mut block = Block::default()
            .title(title)
            .borders(Borders::LEFT | Borders::BOTTOM | Borders::TOP)
            .border_type(BorderType::Plain);

        if is_focused {
            block = block.border_style(Style::default().fg(Color::LightBlue));
        }

        let playlists: Vec<_> = self
            .shown
            .items
            .iter()
            .map(|&i| ListItem::new(self.playlists[i].as_str()))
            .collect();

        let widget = List::new(playlists)
            .block(block)
            .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::Black));
        frame.render_stateful_widget(widget, chunk, &mut self.shown.state);
    }

    #[allow(clippy::single_match)]
    pub fn handle_event(&mut self, app: &mut App, event: Event) -> Result<(), Box<dyn Error>> {
        use crate::command::Command::*;
        use Event::*;
        use KeyCode::*;

        match event {
            Command(cmd) => match cmd {
                SelectNext => self.select_next(app),
                SelectPrev => self.select_prev(app),
                _ => {}
            },
            Terminal(event) => match event {
                crossterm::event::Event::Key(event) => {
                    if !self.filter.is_empty() && self.handle_filter_key_event(event)? {
                        self.refresh_shown();
                        app.channel.send(Event::ChangedPlaylist).unwrap();
                        return Ok(());
                    }

                    match event.code {
                        Up => self.select_prev(app),
                        Down => self.select_next(app),
                        Char('/') => self.filter = "/".into(),
                        _ => {}
                    }
                }
                crossterm::event::Event::Mouse(event) => match event.kind {
                    MouseEventKind::ScrollUp => self.select_prev(app),
                    MouseEventKind::ScrollDown => self.select_next(app),
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    pub fn handle_filter_key_event(&mut self, event: KeyEvent) -> Result<bool, Box<dyn Error>> {
        match event.code {
            KeyCode::Char(c) => {
                self.filter.push(c);
                Ok(true)
            }
            KeyCode::Backspace => {
                self.filter.pop();
                Ok(true)
            }
            KeyCode::Esc | KeyCode::Enter => {
                self.filter.clear();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn select_next(&mut self, app: &mut App) {
        self.shown.select_next();
        app.channel.send(Event::ChangedPlaylist).unwrap();
    }

    pub fn select_prev(&mut self, app: &mut App) {
        self.shown.select_prev();
        app.channel.send(Event::ChangedPlaylist).unwrap();
    }

    pub fn selected_item(&self) -> Option<&str> {
        self.shown
            .selected_item()
            .and_then(|i| self.playlists.get(i))
            .map(|s| s.as_str())
    }

    pub fn open_editor_for_selected(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(selected) = self.selected_item() {
            crate::app::reset_terminal().unwrap();

            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            std::process::Command::new(editor)
                .arg(
                    // playlists_dir/selected.m3u8
                    PathBuf::from(&Config::global().playlists_dir)
                        .join(format!("{}.m3u8", selected)),
                )
                .status()
                .expect("Failed to execute editor");
            self.reload_from_dir()?;

            crate::app::setup_terminal().unwrap();
        }
        Ok(())
    }

    pub fn mode(&self) -> Mode {
        if self.filter.is_empty() {
            Mode::Normal
        } else {
            Mode::Insert
        }
    }
}
