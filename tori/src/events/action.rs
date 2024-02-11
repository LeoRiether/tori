use super::Command;

#[derive(Debug, Default, Clone)]
pub enum Level {
    #[default]
    Ok,
    Info,
    Error,
}

#[derive(Debug, Clone)]
pub enum Action {
    Rerender,
    Tick,
    Command(Command),

    Notify(Level, String),

    ScrollDown,
    ScrollUp,

    Play(String),
    RefreshSongs,
    RefreshPlaylists,
    SelectAndMaybePlaySong(usize),
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
