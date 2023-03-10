use std::error::Error;
use std::io::{self, BufRead, BufReader, ErrorKind, Read, Seek, Write};
use std::mem;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Default, Clone)]
pub struct Song {
    pub title: String,
    pub duration: Duration,
    pub path: String,
}

enum Ext {
    Extm3u,
    Extinf(Duration, String),
}

impl Song {
    pub fn parse_m3u<R: Read>(reader: R) -> Vec<Song> {
        let reader = BufReader::new(reader);
        let mut title = String::default();
        let mut duration = Duration::default();

        let mut songs = Vec::new();
        for line in reader.lines() {
            let line = line.unwrap();
            let line = line.trim();

            // Do nothing
            if line.is_empty() {

                // Parse EXT
            } else if line.starts_with('#') {
                use Ext::*;
                match Song::parse_m3u_extline(line) {
                    Extm3u => {}
                    Extinf(d, t) => {
                        duration = d;
                        title = t;
                    }
                }

            // Parse song path/url
            } else {
                songs.push(Song {
                    title: mem::take(&mut title),
                    duration: mem::take(&mut duration),
                    path: line.into(),
                });
            }
        }
        songs
    }

    fn parse_m3u_extline(line: &str) -> Ext {
        use Ext::*;
        if line.starts_with("#EXTM3U") {
            return Extm3u;
        }

        if let Some(line) = line.strip_prefix("#EXTINF:") {
            let mut parts = line.splitn(2, ',');
            let duration =
                Duration::from_secs(parts.next().unwrap().parse::<f64>().unwrap() as u64);
            let title = parts.next().unwrap().to_string();
            return Extinf(duration, title);
        }

        // TODO: improve error handling here
        panic!("Unknown extline: {}", line);
    }

    /// Parse a song from a given path. The path can be a url or a local file.
    pub fn from_path(path: &str) -> Result<Song, Box<dyn Error>> {
        if path.starts_with("http://") || path.starts_with("https://") {
            Song::parse_ytdlp(path)
        } else {
            Song::parse_local_file(path)
        }
    }

    /// Parses the song using yt-dlp
    pub fn parse_ytdlp(url: &str) -> Result<Song, Box<dyn Error>> {
        // TODO: maybe the user doesn't want to use yt-dlp?
        let output = std::process::Command::new("yt-dlp")
            .arg("--dump-json")
            .arg(url)
            .output()?;

        if !output.status.success() {
            return Err(format!("yt-dlp exited with status {}", output.status).into());
        }

        let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let title = metadata["title"].as_str().unwrap().into();
        let duration = Duration::from_secs(metadata["duration"].as_f64().unwrap() as u64);
        Ok(Song {
            title,
            duration,
            path: url.into(),
        })
    }

    /// Parses song from a local file using audiotags.
    /// Currently audiotags only supports mp3, flac and MPEG-4 (m4a/mp4/...)
    pub fn parse_local_file(path: &str) -> Result<Song, Box<dyn Error>> {
        use audiotags::{
            Error::{ReadError, UnknownFileExtension, UnsupportedFormat, UnsupportedMimeType},
            Tag,
        };

        let default_title = || {
            path.trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("Unknown title")
                .to_string()
        };

        let (title, duration) = match Tag::new().read_from_path(path) {
            Ok(tag) => {
                let title = match (tag.artist(), tag.title()) {
                    (Some(artist), Some(title)) => format!("{} - {}", artist, title),
                    (Some(artist), None) => format!("{} - ?", artist),
                    (None, Some(title)) => title.to_string(),
                    (None, None) => default_title(),
                };
                let duration = tag
                    .duration()
                    .map(Duration::from_secs_f64)
                    .unwrap_or_default();
                (title, duration)
            }
            Err(UnknownFileExtension(_))
            | Err(UnsupportedFormat(_))
            | Err(UnsupportedMimeType(_))
            | Err(ReadError { source: _ }) => (default_title(), Duration::default()),
            Err(e) => return Err(Box::new(e)),
        };

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

    pub fn add_to_playlist(&self, playlist_name: &str) -> Result<(), Box<dyn Error>> {
        let path = PathBuf::from("playlists").join(format!("{}.m3u", playlist_name));
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
