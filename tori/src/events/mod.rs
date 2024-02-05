pub mod channel;
pub mod command;

use crate::config::Config;
pub use command::Command;
use crossterm::event::{Event as CrosstermEvent, KeyEvent};

#[derive(Debug, Clone)]
pub enum Event {
    Tick,
    Action(Action),
    Command(Command),
    Terminal(CrosstermEvent),
}

#[derive(Debug, Clone)]
pub enum Action {
    SongAdded { playlist: String, song: String },
    RefreshSongs,
    SelectSong(usize),
    SelectPlaylist(usize),
}

/// Transforms a key event into the corresponding command, if there is one.
/// Assumes state is in normal mode
pub fn transform_normal_mode_key(key_event: KeyEvent) -> Event {
    use crossterm::event::Event::Key;
    use Event::*;
    match Config::global().keybindings.get_from_event(key_event) {
        Some(cmd) if cmd != command::Command::Nop => Command(cmd),
        _ => Terminal(Key(key_event)),
    }
}
