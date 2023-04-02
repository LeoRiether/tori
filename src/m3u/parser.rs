use std::{
    error::Error,
    fs,
    io::{self, BufRead, BufReader, Read},
    time::Duration,
};

use super::Song;
use super::StringReader;

#[derive(Debug, PartialEq)]
enum Ext {
    Extm3u,
    Extinf(Duration, String),
}

//////////////////////////////
//        LineReader        //
//////////////////////////////
pub trait LineReader {
    fn next_line(&mut self) -> Result<(String, usize), io::Error>;
}

impl<R: Read> LineReader for BufReader<R> {
    fn next_line(&mut self) -> Result<(String, usize), io::Error> {
        let mut buf = String::new();
        let count = self.read_line(&mut buf)?;
        Ok((buf, count))
    }
}

//////////////////////////
//        Parser        //
//////////////////////////
pub struct Parser<L: LineReader> {
    reader: L,
    line_buf: Option<String>,
    cursor: usize,
}

impl Parser<BufReader<fs::File>> {
    pub fn from_file(reader: fs::File) -> Self {
        Self {
            reader: BufReader::new(reader),
            line_buf: None,
            cursor: 0,
        }
    }

    pub fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self, Box<dyn Error>> {
        let file = fs::File::open(path)?;
        Ok(Self::from_file(file))
    }
}

impl<'s> Parser<StringReader<'s>> {
    pub fn from_string(s: &'s str) -> Self {
        Self {
            reader: StringReader::new(s),
            line_buf: None,
            cursor: 0,
        }
    }
}

impl<R: Read> Parser<BufReader<R>> {
    pub fn from_reader(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            line_buf: None,
            cursor: 0,
        }
    }
}

impl<L: LineReader> Parser<L> {
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    fn peek_line(&mut self) -> Result<Option<&str>, Box<dyn Error>> {
        if self.line_buf.is_none() {
            let (mut line, bytes) = self.reader.next_line()?;
            if bytes == 0 {
                return Ok(None);
            }

            self.cursor += bytes;

            let is_nl = |c| c == Some(b'\n') || c == Some(b'\r');
            while is_nl(line.as_bytes().last().copied()) {
                line.pop();
            }
            self.line_buf = Some(line);
        }

        Ok(self.line_buf.as_deref())
    }

    fn consume_line(&mut self) -> Result<Option<String>, Box<dyn Error>> {
        self.peek_line()?;
        Ok(self.line_buf.take())
    }

    pub fn next_header(&mut self) -> Result<bool, Box<dyn Error>> {
        match self.peek_line()? {
            Some(line) if line.starts_with("#EXTM3U") => {
                self.consume_line()?;
                Ok(true)
            }
            _otherwise => Ok(false),
        }
    }

    pub fn next_song(&mut self) -> Result<Option<Song>, Box<dyn Error>> {
        let mut song = Song::default();
        while let Some(line) = self.consume_line()? {
            let line = line.trim();
            if line.is_empty() {
            } else if line.starts_with('#') {
                use Ext::*;
                match parse_extline(line)? {
                    Extm3u => {}
                    Extinf(d, t) => {
                        song.duration = d;
                        song.title = t;
                    }
                }
            } else {
                song.path = line.into();
                return Ok(Some(song));
            }
        }
        Ok(None)
    }

    pub fn all_songs(&mut self) -> Result<Vec<Song>, Box<dyn Error>> {
        let mut songs = Vec::new();
        while let Some(song) = self.next_song()? {
            songs.push(song);
        }
        Ok(songs)
    }
}

fn parse_extline(line: &str) -> Result<Ext, Box<dyn Error>> {
    use Ext::*;
    if line.starts_with("#EXTM3U") {
        return Ok(Extm3u);
    }

    if let Some(line) = line.strip_prefix("#EXTINF:") {
        let mut parts = line.splitn(2, ',');
        let duration = Duration::from_secs(
            parts
                .next()
                .and_then(|p| p.parse::<f64>().ok())
                .unwrap_or_default() as u64,
        );
        let title = parts.next().unwrap_or_default().to_string();
        return Ok(Extinf(duration, title));
    }

    // TODO: improve error handling here
    Err(format!("Unknown extline: {}", line).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extline_parsing() {
        assert_eq!(parse_extline("#EXTM3U").ok(), Some(Ext::Extm3u));
        assert_eq!(
            parse_extline("#EXTINF:10,Artist - Title").ok(),
            Some(Ext::Extinf(
                Duration::from_secs_f64(10.),
                "Artist - Title".into()
            ))
        );
        assert_eq!(
            parse_extline("#EXTINF:").ok(),
            Some(Ext::Extinf(Duration::default(), String::default()))
        );
    }

    #[test]
    fn test_parser() {
        let mut parser = Parser::from_string(
            r#"
            #EXTM3U

            #EXTINF:10,Artist - Title
            https://www.youtube.com/watch?v=dQw4w9WgXcQ
            #EXTINF:0,Yup
            /path/to/local/song
            "#,
        );

        use super::Song;
        assert_eq!(
            parser.all_songs().ok(),
            Some(vec![
                Song {
                    title: "Artist - Title".into(),
                    duration: Duration::from_secs_f64(10.),
                    path: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".into()
                },
                Song {
                    title: "Yup".into(),
                    duration: Duration::from_secs_f64(0.),
                    path: "/path/to/local/song".into()
                }
            ]),
        );
    }
}
