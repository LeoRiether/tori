use super::Action;
use crossterm::event::Event as TermEvent;
use std::{
    sync::{Arc, Mutex},
    thread, time,
};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

pub type Tx = UnboundedSender<Action>;
pub type Rx = UnboundedReceiver<Action>;

/// Channel creates a tokio mpsc channel and spawns two event emitters: one for user input events,
/// another for ticks that happen every second
pub struct Channel {
    pub tx: Tx,
    pub rx: Rx,

    pub crossterm_rx: UnboundedReceiver<TermEvent>,

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
        let (crossterm_tx, crossterm_rx) = unbounded_channel();
        let receiving_crossterm = Arc::new(Mutex::new(()));

        spawn_terminal_event_getter(crossterm_tx.clone(), receiving_crossterm.clone());
        spawn_ticks(tx.clone());

        Self {
            tx,
            rx,
            crossterm_rx,
            receiving_crossterm,
        }
    }
}

fn spawn_terminal_event_getter(
    tx: UnboundedSender<TermEvent>,
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
            tx.send(event).unwrap();
        }
    })
}

fn spawn_ticks(tx: Tx) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(time::Duration::from_secs(1)).await;
            let sent = tx.send(Action::Tick);

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