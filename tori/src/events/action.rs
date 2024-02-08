use super::Command;

#[derive(Debug, Clone)]
pub enum Action {
    Rerender,
    Tick,
    ScrollDown,
    ScrollUp,
    Command(Command),
    SongAdded { playlist: String, song: String },
    RefreshSongs,
    RefreshPlaylists,
    SelectSong(usize),
    SelectPlaylist(usize),
}
