use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

use crate::m3u;

pub enum ToriEvent {
    SongAdded {
        playlist_name: String,
        song: m3u::Song,
    },
}

pub enum Event {
    Terminal(crossterm::event::Event),
    Internal(ToriEvent),
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
}
