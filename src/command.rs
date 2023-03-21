use std::str::FromStr;

macro_rules! parseable_enum {
    ($vis:vis enum $enum:ident { $($item:ident),* $(,)? }) => {
        // I have to put the derive here because it doesn't work outside the macro?
        #[derive(Debug, Default, Clone)]
        $vis enum $enum {
            #[default] // ...it's the first item, always...
            $($item),*
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
    pub enum Command {
        Nop,
        Quit,
        SelectNext,
        SelectPrev,
        OpenInBrowser,
        NextSong,
        PrevSong,
        QueueSong,
        SeekForward,
        SeekBackward,
    }
}
