use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum Command {
    #[default]
    Nop,
    Quit,
    SelectNext,
    SelectPrev,
    NextSong,
    PrevSong,
    TogglePause,
    QueueSong,
    SeekForward,
    SeekBackward,
    OpenInBrowser,
    CopyUrl,
    VolumeUp,
    VolumeDown,
    PlayFromModal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        // not sure why serde_yaml puts a newline there ¯\_(ツ)_/¯
        assert_eq!(serde_yaml::to_string(&Command::Quit).unwrap(), "Quit\n");
        assert_eq!(serde_yaml::to_string(&Command::TogglePause).unwrap(), "TogglePause\n");
    }

    #[test]
    fn test_deserialization() {
        assert_eq!(serde_yaml::from_str::<Command>("Quit").unwrap(), Command::Quit);
        assert_eq!(serde_yaml::from_str::<Command>("VolumeUp").unwrap(), Command::VolumeUp);
    }
}
