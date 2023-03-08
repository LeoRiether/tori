use std::{
    error::Error,
    path::{Path, PathBuf},
};

use crate::app::{App, MyBackend};
use crate::m3u;

use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use tui::{
    layout,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};

#[derive(Debug, Default)]
pub struct SongsPane {
    title: String,
    songs: Vec<m3u::Song>,
    list_state: ListState,
}

impl SongsPane {
    pub fn new() -> Self {
        Self {
            title: " songs ".into(),
            ..Default::default()
        }
    }

    pub fn from_playlist_named(name: &str) -> Self {
        Self::from_playlist({
            let filename = format!("{}.m3u", name);
            PathBuf::from("playlists").join(filename)
        })
    }

    pub fn from_playlist<P: AsRef<Path>>(path: P) -> Self {
        // TODO: maybe return Result?
        let file = std::fs::File::open(&path).expect(&format!(
            "Couldn't open playlist file {}",
            path.as_ref().display()
        ));

        let title = path.as_ref().file_stem().unwrap().to_string_lossy();
        let songs = m3u::parse(file);

        let mut list_state = ListState::default();
        if !songs.is_empty() {
            list_state.select(Some(0));
        }

        SongsPane {
            title: format!(" {} ", title),
            songs,
            list_state,
        }
    }

    pub fn render(
        &mut self,
        is_focused: bool,
        frame: &mut Frame<'_, MyBackend>,
        chunk: layout::Rect,
    ) {
        let mut block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

        if is_focused {
            block = block.border_style(Style::default().fg(Color::LightBlue));
        }

        let songlist: Vec<_> = self
            .songs
            .iter()
            .map(|song| ListItem::new(format!(" {}", song.title)))
            .collect();

        let widget = List::new(songlist)
            .block(block)
            .highlight_style(Style::default().bg(Color::LightYellow).fg(Color::Black))
            .highlight_symbol("»");
        frame.render_stateful_widget(widget, chunk, &mut self.list_state);
    }

    pub fn handle_event(&mut self, app: &mut App, event: Event) -> Result<(), Box<dyn Error>> {
        let has_mod = |event: event::KeyEvent, mods: KeyModifiers| {
            event.modifiers & mods != KeyModifiers::NONE
        };

        match event {
            Event::Key(event) => match event.code {
                KeyCode::Enter if has_mod(event, KeyModifiers::SHIFT) => {
                    if let Some(song) = self.selected_item() {
                        app.mpv
                            .playlist_load_files(&[(
                                &song.path,
                                libmpv::FileState::AppendPlay,
                                None,
                            )])
                            .unwrap();
                    }
                }
                KeyCode::Enter => {
                    if let Some(song) = self.selected_item() {
                        app.mpv
                            .playlist_load_files(&[(&song.path, libmpv::FileState::Replace, None)])
                            .unwrap();
                    }
                }
                // yank, like in vim
                KeyCode::Char('y') => {
                    if let Some(song) = self.selected_item() {
                        let mut ctx: ClipboardContext = ClipboardProvider::new()?;
                        ctx.set_contents(song.path.clone())?;
                    }
                }
                KeyCode::Up => self.select_prev(),
                KeyCode::Down => self.select_next(),
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    pub fn select_next(&mut self) {
        self.list_state.select(match self.list_state.selected() {
            Some(x) => Some(wrap_inc(x, self.songs.len())),
            None => Some(0),
        });
    }

    pub fn select_prev(&mut self) {
        self.list_state.select(match self.list_state.selected() {
            Some(x) => Some(wrap_dec(x, self.songs.len())),
            None => Some(0),
        });
    }

    pub fn selected_item(&self) -> Option<&m3u::Song> {
        self.list_state.selected().map(|i| &self.songs[i])
    }
}

// TODO: uhm I just copied this code from playlists.rs
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
