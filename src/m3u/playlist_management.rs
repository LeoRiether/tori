use std::{
    result::Result as StdResult,
    fs,
    io::{self, Write},
    thread,
};

use crate::{error::Result, app::App, config::Config, events::Event, m3u};

/// Adds a song to an existing playlist
pub fn add_song(app: &mut App, playlist: &str, song_path: String) {
    app.notify_info(format!("Adding {}...", song_path));
    let sender = app.channel.sender.clone();
    let playlist = playlist.to_string();
    thread::spawn(move || {
        add_song_recursively(&song_path, &playlist);

        // Extract last part (separated by '/') of the song_path
        let mut rsplit = song_path.trim_end_matches('/').rsplit('/');
        let song = rsplit.next().unwrap_or(&song_path).to_string();

        let event = Event::SongAdded { playlist, song };
        sender.send(event).expect("Failed to send internal event");
    });
}

/// Adds songs from some path. If the path points to a directory, it'll traverse the directory
/// recursively, adding all songs inside it. If the path points to a file, it'll add that file.
/// If it points to a URL, it adds the url.
/// We do not traverse symlinks, to avoid infinite loops.
fn add_song_recursively(path: &str, playlist_name: &str) {
    let file = std::path::Path::new(&path);
    if file.is_dir() && !file.is_symlink() {
        let mut entries: Vec<_> = std::fs::read_dir(path)
            .expect("Failed to read dir")
            .map(|entry| entry.expect("Failed to read entry").path())
            .collect();

        entries.sort();

        for path in entries {
            let path = path.to_str().expect("Failed to convert path to str");
            add_song_recursively(path, playlist_name);
        }
    } else if !image_file(file) {
        let song = m3u::Song::from_path(path).expect("Failed to parse song");
        song.add_to_playlist(playlist_name)
            .unwrap_or_else(|e| panic!("Failed to add '{}' to playlist. Error: {}", path, e));
    }
}

fn image_file(file: &std::path::Path) -> bool {
    matches!(
        file.extension().and_then(|s| s.to_str()),
        Some("png") | Some("jpg") | Some("jpeg") | Some("webp") | Some("svg")
    )
}

#[derive(Debug)]
pub enum CreatePlaylistError {
    PlaylistAlreadyExists,
    InvalidChar(char),
    IOError(io::Error),
}

impl From<io::Error> for CreatePlaylistError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

/// Creates the corresponding .m3u8 file for a new playlist
pub fn create_playlist(playlist_name: &str) -> StdResult<(), CreatePlaylistError> {
    if playlist_name.contains('/') {
        return Err(CreatePlaylistError::InvalidChar('/'));
    }
    if playlist_name.contains('\\') {
        return Err(CreatePlaylistError::InvalidChar('\\'));
    }

    let path = Config::playlist_path(playlist_name);

    // TODO: when it's stabilized, use std::fs::File::create_new
    if path.try_exists()? {
        Err(CreatePlaylistError::PlaylistAlreadyExists)
    } else {
        std::fs::File::create(path)?;
        Ok(())
    }
}

pub fn delete_song(playlist_name: &str, index: usize) -> Result<()> {
    let path = Config::playlist_path(playlist_name);
    let content = fs::read_to_string(&path)?;
    let mut parser = m3u::Parser::from_string(&content);

    parser.next_header()?;
    for _ in 0..index {
        parser.next_song()?;
    }

    let start_pos = parser.cursor();
    let _song = parser.next_song()?;
    let end_pos = parser.cursor();

    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)?;
    file.write_all(content[..start_pos].as_bytes())?;
    file.write_all(content[end_pos..].as_bytes())?;

    Ok(())
}

pub fn rename_song(
    playlist_name: &str,
    index: usize,
    new_name: &str,
) -> Result<()> {
    let path = Config::playlist_path(playlist_name);
    let content = fs::read_to_string(&path)?;
    let mut parser = m3u::Parser::from_string(&content);

    parser.next_header()?;
    for _ in 0..index {
        parser.next_song()?;
    }

    let start_pos = parser.cursor();
    let song = parser.next_song()?;
    let end_pos = parser.cursor();

    if let Some(mut song) = song {
        song.title = new_name.to_string();

        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)?;
        file.write_all(content[..start_pos].as_bytes())?;
        file.write_all(song.serialize().as_bytes())?;
        file.write_all(content[end_pos..].as_bytes())?;
    }

    Ok(())
}

/// Swaps `index`-th song with the `index+1`-th (0-indexed)
pub fn swap_song(playlist_name: &str, index: usize) -> Result<()> {
    let path = Config::playlist_path(playlist_name);
    let content = fs::read_to_string(&path)?;
    let mut parser = m3u::Parser::from_string(&content);

    parser.next_header()?;
    for _ in 0..index {
        parser.next_song()?;
    }

    let start_pos = parser.cursor();
    let song1 = parser.next_song()?;
    let song2 = parser.next_song()?;
    let end_pos = parser.cursor();

    if let (Some(song1), Some(song2)) = (song1, song2) {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)?;
        file.write_all(content[..start_pos].as_bytes())?;
        file.write_all(song2.serialize().as_bytes())?;
        file.write_all(song1.serialize().as_bytes())?;
        file.write_all(content[end_pos..].as_bytes())?;
    }

    Ok(())
}

pub fn delete_playlist(playlist_name: &str) -> Result<()> {
    let path = Config::playlist_path(playlist_name);
    fs::remove_file(path)?;
    Ok(())
}
