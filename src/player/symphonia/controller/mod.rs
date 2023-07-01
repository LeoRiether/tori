use super::source::{file_source, ytdlp_source};

use crate::error::Result;

#[derive(Debug, Default)]
pub struct Controller {}

impl Controller {
    pub fn play(&mut self, path: &str) -> Result<()> {
        start_player_thread(path);
        Ok(())
    }
}

fn start_player_thread(mut path: &str) {
    let force_ytdlp;
    if let Some(url) = path.strip_prefix("ytdlp://") {
        path = url;
        force_ytdlp = true;
    } else {
        force_ytdlp = false;
    };

    if force_ytdlp || path.starts_with("http://") || path.starts_with("https://") {
        ytdlp_source::start_player_thread(path);
    } else {
        file_source::start_player_thread(path);
    }
}
