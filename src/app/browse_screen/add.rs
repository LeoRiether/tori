use std::{error::Error, thread};

use tui::Frame;

use crate::{
    app::{
        popup::{self, Popup},
        App, Mode, MyBackend,
    },
    events::Event,
    m3u,
};

#[derive(Debug, Default)]
pub struct AddPane {
    popup: Popup,
    playlist: String,
}

impl AddPane {
    pub fn new(playlist: String) -> Self {
        Self {
            popup: Popup::new("Add song"),
            playlist,
        }
    }

    pub fn handle_event(&mut self, app: &mut App, event: Event) -> Result<(), Box<dyn Error>> {
        match self.popup.handle_event(event)? {
            popup::Message::Nothing => {}
            popup::Message::Quit => {}
            popup::Message::Commit(path) => commit(path, app, &self.playlist),
        }
        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame<'_, MyBackend>) {
        self.popup.render(frame)
    }

    pub fn mode(&self) -> Mode {
        Mode::Insert
    }
}

/// Adds the song to the current playlist
fn commit(path: String, app: &mut App, playlist: &str) {
    app.notify_info(format!("Adding {}...", path));
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
