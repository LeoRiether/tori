use std::{
    borrow::Cow,
    error::Error,
    io::{BufRead, BufReader, Read},
    time::Duration,
};

use super::Song;

enum Ext {
    Extm3u,
    Extinf(Duration, String),
}

pub struct Parser<R: Read> {
    reader: BufReader<R>,
    line_buf: Option<String>,
}

impl<R: Read> Parser<R> {
    pub fn from_reader(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            line_buf: None,
        }
    }

    fn consume_line(&mut self) -> Result<Option<String>, Box<dyn Error>> {
        self.peek_line();
        Ok(self.line_buf.take())
    }

    fn peek_line(&mut self) -> Result<Option<&str>, Box<dyn Error>> {
        if self.line_buf.is_none() {
            let mut line = String::new();
            let bytes = self.reader.read_line(&mut line)?;
            if bytes == 0 {
                return Ok(None);
            }

            let is_nl = |c| c == Some(&b'\n') || c == Some(&b'\r');
            while is_nl(line.as_bytes().last()) {
                line.pop();
            }
            self.line_buf = Some(line);
        }

        Ok(self.line_buf.as_deref())
    }

    pub fn next_header(&mut self) -> Result<bool, Box<dyn Error>> {
        match self.peek_line()? {
            Some(line) if line.starts_with("#EXTM3U") => {
                self.consume_line();
                Ok(true)
            }
            _otherwise => Ok(false),
        }
    }

    pub fn next_song(&mut self) -> Result<Song, Box<dyn Error>> {
        todo!("Read next song")
    }
}

fn parse_m3u_extline(line: &str) -> Ext {
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
