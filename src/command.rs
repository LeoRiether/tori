use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Hash, Serialize, Deserialize)]
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
    Shuffle,
    Rename,

    /// Select next item (like a song or playlist)
    SelectNext,

    /// Select previous item (like a song or playlist)
    SelectPrev,

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
