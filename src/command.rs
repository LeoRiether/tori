use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
pub enum Command {
    #[default]
    Nop,
    Quit,
    NextSong,
    PrevSong,
    TogglePause,
    SeekForward,
    SeekBackward,
    OpenInBrowser,
    CopyUrl,
    CopyTitle,
    VolumeUp,
    VolumeDown,
    Mute,
    ToggleVisualizer,
    NextSortingMode,

    /// Rename selected song or playlist
    Rename,

    /// Delete selected song or playlist
    Delete,

    /// Swap the selected song with the one below it
    SwapSongDown,

    /// Swap the selected song with the one above it
    SwapSongUp,

    /// Shuffle current playlist
    Shuffle,

    /// Select next item (like a song or playlist)
    SelectNext,

    /// Select previous item (like a song or playlist)
    SelectPrev,

    /// Select the pane to the right (the same as pressing the \<right> key)
    SelectRight,

    /// Select the pane to the left (the same as pressing the \<left> key)
    SelectLeft,

    /// Add a new song or playlist
    Add,

    /// Add song to the queue
    QueueSong,

    /// Add all shown songs to the queue
    QueueShown,

    /// Queries the user for a song to play, without adding it to a playlist
    PlayFromModal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        // not sure why serde_yaml puts a newline there ¯\_(ツ)_/¯
        assert_eq!(serde_yaml::to_string(&Command::Quit).unwrap(), "Quit\n");
        assert_eq!(
            serde_yaml::to_string(&Command::TogglePause).unwrap(),
            "TogglePause\n"
        );
    }

    #[test]
    fn test_deserialization() {
        assert_eq!(
            serde_yaml::from_str::<Command>("Quit").unwrap(),
            Command::Quit
        );
        assert_eq!(
            serde_yaml::from_str::<Command>("VolumeUp").unwrap(),
            Command::VolumeUp
        );
    }
}
