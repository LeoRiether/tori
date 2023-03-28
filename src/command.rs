use std::str::FromStr;

macro_rules! parseable_enum {
    (
        $( #[$attr:meta] )*
        $vis:vis enum $enum:ident { $($(#[$item_attr:meta])* $item:ident),* $(,)? }
    ) => {
        $( #[$attr] )*
        $vis enum $enum {
            $(
                $(#[$item_attr])*
                $item
            ),*
        }

        impl FromStr for $enum {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(stringify!($item) => Ok($enum::$item)),*,
                    _ => Err(format!("No such variant: {}", s)),
                }
            }
        }

        impl From<$enum> for String {
            fn from(e: $enum) -> String {
                match e {
                    $($enum::$item => stringify!($item).to_string()),*,
                }
            }
        }
    };
}

parseable_enum! {
    #[derive(Debug, Default, Clone)]
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
}
