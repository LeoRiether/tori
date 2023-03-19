use std::time;
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

#[derive(Debug, Clone)]
pub enum Command {
    Quit,
}

#[derive(Debug, Clone)]
pub enum Event {
    SecondTick,
    SongAdded {
        playlist: String,
        song: String, 
    },
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
