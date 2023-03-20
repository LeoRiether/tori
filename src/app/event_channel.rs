use std::str::FromStr;
use std::time;
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

macro_rules! parseable_enum {
    ($vis:vis enum $enum:ident { $($item:ident),* $(,)? }) => {
        // I have to put the derive here because it doesn't work outside the macro?
        #[derive(Debug, Clone)]
        $vis enum $enum {
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
        Quit,
        SelectNext,
        SelectPrev,
        OpenInBrowser,
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    SecondTick,
    SongAdded { playlist: String, song: String },
    ChangedPlaylist,
    Command(Command),
    Terminal(crossterm::event::Event),
}

pub struct Channel {
    pub sender: Sender<Event>,
    pub receiver: Receiver<Event>,
}

impl Default for Channel {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }
}

impl Channel {
    pub fn spawn_terminal_event_getter(&self) {
        let sender = self.sender.clone();
        thread::spawn(move || loop {
            if let Ok(event) = crossterm::event::read() {
                sender.send(Event::Terminal(event)).unwrap();
            }
        });
    }

    pub fn spawn_ticks(&self) {
        let sender = self.sender.clone();
        thread::spawn(move || loop {
            thread::sleep(time::Duration::from_secs(1));
            sender.send(Event::SecondTick).unwrap();
        });
    }
}
