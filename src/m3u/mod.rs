use std::io::{self, ErrorKind, Read, Seek, Write};

use std::time::Duration;

use crate::{config::Config, error::Result};

pub mod stringreader;
pub use stringreader::StringReader;

pub mod parser;
pub use parser::Parser;

pub mod playlist_management;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Song {
    pub title: String,
    pub duration: Duration,
    pub path: String,
}

impl Song {
    /// Parse a song from a given path. The path can be a url or a local file.
    pub fn from_path(path: &str) -> Result<Song> {
        if let Some(path) = path.strip_prefix("ytdl://") {
            let mut song = Song::parse_ytdlp(path)?;
            song.path = format!("ytdl://{}", song.path);
            Ok(song)
        } else if path.starts_with("http://") || path.starts_with("https://") {
            Song::parse_ytdlp(path)
        } else {
            Song::parse_local_file(path)
        }
    }

    /// Parses the song using yt-dlp
    pub fn parse_ytdlp(url: &str) -> Result<Song> {
        // TODO: maybe the user doesn't want to use yt-dlp?
        let output = std::process::Command::new("yt-dlp")
            .arg("--dump-single-json")
            .arg("--flat-playlist")
            .arg(url)
            .output()
            .map_err(|e| format!("Could not execute yt-dlp. Error: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "yt-dlp exited with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let title = metadata["title"].as_str().unwrap_or("?").into();
        let duration = Duration::from_secs_f64(metadata["duration"].as_f64().unwrap_or(0.0));
        Ok(Song {
            title,
            duration,
            path: url.into(),
        })
    }

    /// Parses song from a local file using lofty.
    pub fn parse_local_file(path: &str) -> Result<Song> {
        use lofty::{
            error::ErrorKind::{NotAPicture, UnknownFormat, UnsupportedPicture, UnsupportedTag},
            Accessor, AudioFile, TaggedFileExt,
        };

        let default_title = || {
            path.trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("Unknown title")
                .to_string()
        };

        let tagged_file = match lofty::read_from_path(path) {
            Ok(tf) => Some(tf),
            Err(e) => match e.kind() {
                UnknownFormat | NotAPicture | UnsupportedPicture | UnsupportedTag => None,
                _ => return Err(e.into()),
            },
        };

        let tag = tagged_file
            .as_ref()
            .and_then(|t| t.primary_tag().or(t.first_tag()));

        let title = match (
            tag.and_then(Accessor::artist),
            tag.and_then(Accessor::title),
        ) {
            (Some(artist), Some(title)) => format!("{} - {}", artist, title),
            (Some(artist), None) => format!("{} - ?", artist),
            (None, Some(title)) => title.to_string(),
            (None, None) => default_title(),
        };

        let duration = tagged_file
            .map(|t| t.properties().duration())
            .unwrap_or_default();

        Ok(Song {
            title,
            duration,
            path: path.into(),
        })
    }

    pub fn serialize(&self) -> String {
        let duration = self.duration.as_secs();
        format!("#EXTINF:{},{}\n{}\n", duration, self.title, self.path)
    }

    pub fn add_to_playlist(&self, playlist_name: &str) -> Result<()> {
        let path = Config::playlist_path(playlist_name);
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path)?;

        // Write a #EXTM3U header if it doesn't exist
        file.rewind()?;
        let mut buf = vec![0u8; "#EXTM3U".len()];
        let has_extm3u = match file.read_exact(&mut buf) {
            Ok(()) if buf == b"#EXTM3U" => true,
            Ok(()) => false,
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => false,
            Err(e) => {
                return Err(Box::new(e));
            }
        };

        if !has_extm3u {
            let mut buf = Vec::new();
            file.rewind()?;
            file.read_to_end(&mut buf)?;
            file.set_len(0)?;
            file.write_all(b"#EXTM3U\n")?;
            file.write_all(&buf)?;
        }

        // Check if the file ends in a newline
        file.seek(io::SeekFrom::End(-1))?;
        buf = vec![0u8];
        let ends_with_newline = match file.read_exact(&mut buf) {
            Ok(()) if buf == b"\n" => true,
            Ok(()) => false,
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => false,
            Err(e) => {
                return Err(Box::new(e));
            }
        };

        if !ends_with_newline {
            file.write_all(b"\n")?;
        }

        // Write the serialized song
        file.write_all(self.serialize().as_bytes())?;
        Ok(())
    }
}
