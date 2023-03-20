use std::{error::Error, mem, thread};

use crossterm::event::KeyCode;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    app::{event_channel::Event, App, Mode, MyBackend},
    m3u,
};

#[derive(Debug, Default)]
pub struct AddPane {
    pub path: String,
}

impl AddPane {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::single_match)]
    pub fn handle_event(
        &mut self,
        app: &mut App,
        selected_playlist: Option<&str>,
        event: Event,
    ) -> Result<(), Box<dyn Error>> {
        use Event::*;
        use KeyCode::*;
        match event {
            Terminal(crossterm::event::Event::Key(event)) => match event.code {
                Char(c) => self.path.push(c),
                Backspace => {
                    self.path.pop();
                }
                Esc => {
                    self.path.clear();
                }
                Enter => {
                    if let Some(playlist) = selected_playlist {
                        self.commit(app, playlist);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame<'_, MyBackend>) {
        let size = frame.size();
        let chunk = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(48),
                Constraint::Length(5),
                Constraint::Percentage(48),
            ])
            .split(size)[1];
        let chunk = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Ratio(1, 4),
                Constraint::Ratio(2, 4),
                Constraint::Ratio(1, 4),
            ])
            .split(chunk)[1];

        let block = Block::default()
            .title(" Add Song ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::LightBlue));

        let paragraph = Paragraph::new(format!("\n{}", self.path))
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(Clear, chunk);
        frame.render_widget(paragraph, chunk);
    }

    /// Adds the song to the current playlist
    fn commit(&mut self, app: &mut App, playlist: &str) {
        let path = mem::take(&mut self.path);
        let sender = app.channel.sender.clone();
        let playlist = playlist.to_string();
        thread::spawn(move || {
            add_recursively(&path, &playlist);

            let mut rsplit = path.trim_end_matches('/').rsplit('/');
            let song = rsplit.next().unwrap_or(&path).to_string();
            let event = Event::SongAdded { playlist, song };
            sender.send(event).expect("Failed to send internal event");
        });
    }

    pub fn mode(&self) -> Mode {
        Mode::Insert
    }
}

/// Adds songs from some path. If the path points to a directory, it'll traverse the directory
/// recursively, adding all songs inside it. If the path points to a file, it'll add that file.
/// If it points to a URL, it adds the url.
/// We do not traverse symlinks, to avoid infinite loops.
fn add_recursively(path: &str, playlist_name: &str) {
    let file = std::path::Path::new(&path);
    if file.is_dir() && !file.is_symlink() {
        for entry in std::fs::read_dir(path).expect("Failed to read dir") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            let path = path.to_str().expect("Failed to convert path to str");
            add_recursively(path, playlist_name);
        }
    } else {
        let song = m3u::Song::from_path(path).expect("Failed to parse song");
        song.add_to_playlist(playlist_name)
            .expect("Failed to add song to playlist");
    }
}
