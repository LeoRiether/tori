use std::{
    fs,
    io::{self, Write},
    path,
    result::Result as StdResult,
};

use crate::{
    config::Config,
    error::Result,
    events::{action::Level, channel::Tx, Action},
    m3u,
};

/// Adds a song to an existing playlist
pub async fn add_song(tx: Tx, playlist: String, song_path: String) -> Result<()> {
    tx.send(Action::Notify(
        Level::Info,
        format!("Adding {}...", song_path),
    ))?;

    if surely_invalid_path(&song_path) {
        return Err(format!("Failed to add song path '{}'. Doesn't look like a URL and is not a valid path in your filesystem.", song_path).into());
    }

    add_song_recursively(&song_path, &playlist);

    // Extract last part (separated by '/') of the song_path
    let mut rsplit = song_path.trim_end_matches('/').rsplit('/');
    let song = rsplit.next().unwrap_or(&song_path).to_string();

    tx.send(Action::SongAdded { playlist, song })?;
    Ok(())
}

/// Adds songs from some path. If the path points to a directory, it'll traverse the directory
/// recursively, adding all songs inside it. If the path points to a file, it'll add that file.
/// If it points to a URL, it adds the url.
/// We do not traverse symlinks, to avoid infinite loops.
fn add_song_recursively(path: &str, playlist_name: &str) {
    let file = std::path::Path::new(&path);
    if file.is_dir() && !file.is_symlink() {
        let mut entries: Vec<_> = fs::read_dir(path)
            .unwrap_or_else(|e| panic!("Failed to read directory '{}'. Error: {}", path, e))
            .map(|entry| entry.expect("Failed to read entry").path())
            .collect();

        entries.sort();

        for path in entries {
            let path = path.to_str().unwrap_or_else(|| {
                panic!(
                    "Failed to add '{}' to playlist. Path is not valid UTF-8",
                    path.display()
                )
            });
            add_song_recursively(path, playlist_name);
        }
    } else if !image_file(file) {
        let song = m3u::Song::from_path(path)
            .unwrap_or_else(|e| panic!("Failed to add '{}' to playlist. Error: {}", path, e));
        song.add_to_playlist(playlist_name)
            .unwrap_or_else(|e| panic!("Failed to add '{}' to playlist. Error: {}", path, e));
    }
}

fn surely_invalid_path(path: &str) -> bool {
    let file = std::path::Path::new(&path);
    !file.is_dir() // not a directory...
        && !file.exists() // ...or a valid filepath...
        && !path.starts_with("http://") // ...or a URL...
        && !path.starts_with("https://")
        && !path.starts_with("ytdl://")
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

#[derive(Debug)]
pub enum RenamePlaylistError {
    PlaylistAlreadyExists,
    EmptyPlaylistName,
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
        fs::File::create(path)?;
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

pub fn rename_song(playlist_name: &str, index: usize, new_name: &str) -> Result<()> {
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

pub fn rename_playlist(playlist_name: &str, new_name: &str) -> StdResult<(), RenamePlaylistError> {
    if new_name.is_empty() {
        return Err(RenamePlaylistError::EmptyPlaylistName);
    }

    if new_name.contains('/') {
        return Err(RenamePlaylistError::InvalidChar('/'));
    }

    if new_name.contains('\\') {
        return Err(RenamePlaylistError::InvalidChar('\\'));
    }

    let old_path: path::PathBuf = Config::playlist_path(playlist_name);
    let new_path: path::PathBuf = Config::playlist_path(new_name);

    if let Ok(metadata) = fs::metadata(new_path.clone()) {
        if metadata.is_file() || metadata.is_dir() {
            return Err(RenamePlaylistError::PlaylistAlreadyExists);
        }
    }

    match fs::rename(old_path, &new_path) {
        Err(e) => Err(RenamePlaylistError::IOError(e)),
        Ok(_) => Ok(()),
    }
}

pub fn delete_playlist(playlist_name: &str) -> Result<()> {
    let path = Config::playlist_path(playlist_name);
    fs::remove_file(path)?;
    Ok(())
}
