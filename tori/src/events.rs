use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use std::{
    sync::{Arc, Mutex},
    thread, time,
};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

use crate::config::Config;

use super::command::Command;

#[derive(Debug, Clone)]
pub enum Event {
    SecondTick,
    SongAdded { playlist: String, song: String },
    ChangedPlaylist,
    Command(Command),
    Terminal(CrosstermEvent),
}

/// Transforms a key event into the corresponding command, if there is one.
/// Assumes state is in normal mode
pub fn transform_normal_mode_key(key_event: KeyEvent) -> Event {
    use crate::command::Command::Nop;
    use crossterm::event::Event::Key;
    use Event::*;
    match Config::global().keybindings.get_from_event(key_event) {
        Some(cmd) if cmd != Nop => Command(cmd),
        _ => Terminal(Key(key_event)),
    }
}

pub struct Channel {
    pub tx: UnboundedSender<Event>,
    pub rx: UnboundedReceiver<Event>,

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
        Self::new()
    }
}

impl Channel {
    pub fn new() -> Self {
        let (tx, rx) = unbounded_channel();
        let receiving_crossterm = Arc::new(Mutex::new(()));

        spawn_terminal_event_getter(tx.clone(), receiving_crossterm.clone());
        spawn_ticks(tx.clone());

        Self {
            tx,
            rx,
            receiving_crossterm,
        }
    }
}

fn spawn_terminal_event_getter(
    tx: UnboundedSender<Event>,
    receiving_crossterm: Arc<Mutex<()>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        {
            // WARNING: very short-lived lock.
            // Otherwise this mutex keeps relocking and starving the other thread.
            // I'm sure this will work in all cases (spoiler: no it doesn't (but maybe it does))
            let _lock = receiving_crossterm.lock().unwrap();
        };

        if let Ok(event) = crossterm::event::read() {
            tx.send(Event::Terminal(event)).unwrap();
        }
    })
}

fn spawn_ticks(tx: UnboundedSender<Event>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(time::Duration::from_secs(1)).await;
            let sent = tx.send(Event::SecondTick);

            // Stop spawning ticks if the receiver has been dropped. This prevents a
            // 'called `Result::unwrap()` on an `Err` value: SendError { .. }' panic when Ctrl+C is
            // pressed and the receiver is dropped right before the sender tries to send the
            // tick
            if sent.is_err() {
                return;
            }
        }
    })
}
