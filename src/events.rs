
use std::sync::{mpsc, Arc, Mutex};
use std::time::{self};
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

use super::command::Command;

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

    /// Whoever owns this mutex can receive events from crossterm::event::read().
    ///
    /// Sometimes, like when the editor is open for editing a playlist, we dont' want to call
    /// `crosssterm::event::read()` because it will intercept the keypresses we want to send to the
    /// editor. In this case, the thread that opened the editor will hold the mutex until the user
    /// closes the editor.
    pub receiving_crossterm: Arc<Mutex<()>>,
}

impl Default for Channel {
    fn default() -> Self {
        let (sender, receiver) = channel();
        let receiving_crossterm = Arc::new(Mutex::new(()));
        Self {
            sender,
            receiver,
            receiving_crossterm,
        }
    }
}

impl Channel {
    pub fn spawn_terminal_event_getter(&self) -> thread::JoinHandle<()> {
        let sender = self.sender.clone();
        let receiving_crossterm = self.receiving_crossterm.clone();
        thread::spawn(move || loop {
            {
                // WARNING: very short-lived lock.
                // Otherwise this mutex keeps relocking and starving the other thread.
                // I'm sure this will work in all cases (spoiler: no it doesn't (but maybe it does)) 
                let _lock = receiving_crossterm.lock().unwrap();
            };

            if let Ok(event) = crossterm::event::read() {
                sender.send(Event::Terminal(event)).unwrap();
            }
        })
    }

    pub fn spawn_ticks(&self) -> thread::JoinHandle<()> {
        let sender = self.sender.clone();
        thread::spawn(move || loop {
            thread::sleep(time::Duration::from_secs(1));
            sender.send(Event::SecondTick).unwrap();
        })
    }

    pub fn send(&mut self, event: Event) -> Result<(), mpsc::SendError<Event>> {
        self.sender.send(event)
    }
}
