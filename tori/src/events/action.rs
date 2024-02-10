use super::Command;

#[derive(Debug, Clone)]
pub enum Action {
    Rerender,
    Tick,
    ScrollDown,
    ScrollUp,
    Command(Command),
    RefreshSongs,
    RefreshPlaylists,
    SelectSong(usize),
    SelectPlaylist(usize),
    CloseModal,
    AddPlaylist {
        name: String,
    },
    RenamePlaylist {
        playlist: String,
        new_name: String,
    },
    DeletePlaylist {
        playlist: String,
    },
    AddSongToPlaylist {
        playlist: String,
        song: String,
    },
    SongAdded {
        playlist: String,
        song: String,
    },
    RenameSong {
        playlist: String,
        index: usize,
        new_name: String,
    },
    DeleteSong {
        playlist: String,
        index: usize,
    },
}
