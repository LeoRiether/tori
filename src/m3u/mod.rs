use std::io::{BufRead, BufReader, Read};
use std::mem;
use std::time::Duration;

#[derive(Debug, Default)]
pub struct Song {
    pub title: String,
    pub duration: Duration,
    pub path: String,
}

pub fn parse<R: Read>(reader: R) -> Vec<Song> {
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
            match parse_extline(line) {
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

enum Ext {
    Extm3u,
    Extinf(Duration, String),
}

fn parse_extline(line: &str) -> Ext {
    use Ext::*;
    if line.starts_with("#EXTM3U") {
        return Extm3u;
    }

    if let Some(line) = line.strip_prefix("#EXTINF:") {
        let mut parts = line.splitn(2, ',');
        let duration = Duration::from_secs(parts.next().unwrap().parse::<f64>().unwrap() as u64);
        let title = parts.next().unwrap().to_string();
        return Extinf(duration, title);
    }

    // TODO: improve error handling here
    panic!("Unknown extline: {}", line);
}
