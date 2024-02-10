use super::Command;
use crate::config::Config;
use crossterm::event::KeyEvent;

#[derive(Debug, Clone)]
pub enum Action {
    Tick,
    Input(KeyEvent),
    Command(Command),
    SongAdded { playlist: String, song: String },
    RefreshSongs,
    SelectSong(usize),
    SelectPlaylist(usize),
}
