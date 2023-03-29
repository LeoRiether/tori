use std::{
    error::Error,
    path::{Path, PathBuf},
};

use crate::app::{filtered_list::FilteredList, App, Mode, MyBackend};
use crate::events::Event;
use crate::m3u;

use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent, MouseEventKind};
use tui::{
    layout::{self, Constraint},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Row, Table, TableState},
    Frame,
};

#[derive(Debug, Default)]
pub struct SongsPane {
    title: String,
    songs: Vec<m3u::Song>,
    shown: FilteredList<TableState>,
    filter: String,
}

impl SongsPane {
    pub fn new() -> Self {
        Self {
            title: " songs ".into(),
            ..Default::default()
        }
    }

    pub fn from_playlist_pane(playlists: &super::playlists::PlaylistsPane) -> Self {
        match playlists.selected_item() {
            Some(playlist) => SongsPane::from_playlist_named(playlist),
            None => SongsPane::new(),
        }
    }

    pub fn from_playlist_named(name: &str) -> Self {
        Self::from_playlist({
            let filename = format!("{}.m3u", name);
            PathBuf::from(&crate::config::Config::global().playlists_dir).join(filename)
        })
    }

    pub fn from_playlist(path: impl AsRef<Path>) -> Self {
        // TODO: maybe return Result?
        let file = std::fs::File::open(&path)
            .unwrap_or_else(|_| panic!("Couldn't open playlist file {}", path.as_ref().display()));

        let title = path
            .as_ref()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let songs = m3u::Song::parse_m3u(file);
        let shown = FilteredList::default();

        let mut me = Self {
            title,
            songs,
            shown,
            filter: String::new(),
        };

        me.refresh_shown();
        me
    }

    fn refresh_shown(&mut self) {
        self.shown.filter(&self.songs, |s| {
            self.filter.is_empty()
                || s.title
                    .to_lowercase()
                    .contains(&self.filter[1..].to_lowercase())
                || s.path
                    .to_lowercase()
                    .contains(&self.filter[1..].to_lowercase())
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
            format!(" {} ", self.title)
        };

        let mut block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

        if is_focused {
            block = block.border_style(Style::default().fg(Color::LightBlue));
        }

        let songlist: Vec<_> = self
            .shown
            .items
            .iter()
            .map(|&i| &self.songs[i])
            .map(|song| {
                Row::new(vec![
                    format!(" {}", song.title),
                    format!(
                        "{}:{:02}",
                        song.duration.as_secs() / 60,
                        song.duration.as_secs() % 60
                    ),
                ])
            })
            .collect();

        let widths = &[Constraint::Length(chunk.width - 11), Constraint::Length(10)];
        let widget = Table::new(songlist)
            .block(block)
            .widths(widths)
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
            .highlight_symbol(" ◇");
        frame.render_stateful_widget(widget, chunk, &mut self.shown.state);
    }

    #[allow(clippy::single_match)]
    pub fn handle_event(&mut self, app: &mut App, event: Event) -> Result<(), Box<dyn Error>> {
        use crate::command::Command::*;
        use Event::*;
        use KeyCode::*;

        match event {
            Command(cmd) => match cmd {
                SelectNext => self.select_next(),
                SelectPrev => self.select_prev(),
                QueueSong => {
                    if let Some(song) = self.selected_item() {
                        app.mpv.playlist_load_files(&[(
                            &song.path,
                            libmpv::FileState::AppendPlay,
                            None,
                        )])?;
                    }
                }
                OpenInBrowser => {
                    if let Some(song) = self.selected_item() {
                        // TODO: reconsider if I really need a library to write this one line
                        webbrowser::open(&song.path)?;
                    }
                }
                CopyUrl => {
                    if let Some(song) = self.selected_item() {
                        let mut ctx: ClipboardContext = ClipboardProvider::new()?;
                        ctx.set_contents(song.path.clone())?;
                        app.notify_info(format!("Copied {} to the clipboard", song.path));
                    }
                }
                CopyTitle => {
                    if let Some(song) = self.selected_item() {
                        let mut ctx: ClipboardContext = ClipboardProvider::new()?;
                        ctx.set_contents(song.title.clone())?;
                        app.notify_info(format!("Copied {} to the clipboard", song.title));
                    }
                }
                _ => {}
            },
            SongAdded {
                playlist: _,
                song: _,
            } => {
                // scroll to the bottom
                if !self.shown.items.is_empty() {
                    self.shown.state.select(Some(self.shown.items.len() - 1));
                }
            }
            Terminal(event) => match event {
                crossterm::event::Event::Key(event) => {
                    if !self.filter.is_empty() && self.handle_filter_key_event(event)? {
                        self.refresh_shown();
                        return Ok(());
                    }

                    match event.code {
                        Enter => {
                            if let Some(song) = self.selected_item() {
                                app.mpv.playlist_load_files(&[(
                                    &song.path,
                                    libmpv::FileState::Replace,
                                    None,
                                )])?;
                            }
                        }
                        // Go to the bottom, also like in vim
                        Char('G') => {
                            if !self.shown.items.is_empty() {
                                self.shown.state.select(Some(self.shown.items.len() - 1));
                            }
                        }
                        Up => self.select_prev(),
                        Down => self.select_next(),
                        Char('/') => self.filter = "/".into(),
                        _ => {}
                    }
                }
                crossterm::event::Event::Mouse(event) => match event.kind {
                    MouseEventKind::ScrollUp => self.select_prev(),
                    MouseEventKind::ScrollDown => self.select_next(),
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    /// Handles a key event when the filter is active.
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
            KeyCode::Esc => {
                self.filter.clear();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn select_next(&mut self) {
        self.shown.select_next();
    }

    pub fn select_prev(&mut self) {
        self.shown.select_prev();
    }

    pub fn selected_item(&self) -> Option<&m3u::Song> {
        self.shown.selected_item().and_then(|i| self.songs.get(i))
    }

    pub fn mode(&self) -> Mode {
        if self.filter.is_empty() {
            Mode::Normal
        } else {
            Mode::Insert
        }
    }
}
