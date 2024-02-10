use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    time::{self, Duration, Instant},
};
use tokio::select;
use tui::{backend::CrosstermBackend, Terminal};

use crate::{
    app::component::Mode, error::Result, events::channel::Channel, state::State, ui::ui,
    update::update,
};

pub mod app_screen;
pub mod browse_screen;
pub mod component;
pub mod filtered_list;
pub mod modal;
pub mod playlist_screen;

use crate::events::Event;

type MyBackend = CrosstermBackend<io::Stdout>;

const FRAME_DELAY_MS: u16 = 16;
const HIGH_EVENT_TIMEOUT: u16 = 1000;
const LOW_EVENT_TIMEOUT: u16 = 17;

/// Controls the application main loop
pub struct App<'n> {
    pub state: State<'n>,
    pub channel: Channel,
    pub terminal: Terminal<MyBackend>,
    next_render: time::Instant,
    next_poll_timeout: u16,
}

impl<'a> App<'a> {
    pub fn new() -> Result<App<'a>> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);

        Ok(App {
            state: State::new()?,
            channel: Channel::new(),
            terminal: Terminal::new(backend)?,
            next_render: time::Instant::now(),
            next_poll_timeout: LOW_EVENT_TIMEOUT,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        chain_hook();
        setup_terminal()?;

        while !self.state.quit {
            self.render()
                .map_err(|e| self.state.notify_err(format!("Rendering error: {e}")))
                .ok();

            if let Some(ev) = self.recv_event().await {
                let tx = self.channel.tx.clone();
                match update(&mut self.state, tx, ev) {
                    Ok(Some(ev)) => self.channel.tx.send(ev).expect("Failed to send event"),
                    Ok(None) => {}
                    Err(e) => self.state.notify_err(e.to_string()),
                }
            }
        }

        reset_terminal()?;
        Ok(())
    }

    #[inline]
    fn render(&mut self) -> Result<()> {
        if Instant::now() >= self.next_render {
            self.terminal.draw(|frame| {
                let area = frame.size();
                let buf = frame.buffer_mut();
                ui(&mut self.state, area, buf);
            })?;

            self.next_render = time::Instant::now()
                .checked_add(Duration::from_millis(FRAME_DELAY_MS as u64))
                .unwrap();
        }
        Ok(())
    }

    async fn recv_event(&mut self) -> Option<Event> {
        // NOTE: Big timeout if the last event was long ago, small timeout otherwise.
        // This makes it so after a burst of events, like a Ctrl+V, we get a small timeout
        // immediately after the last event, which triggers a fast render.
        let timeout = Duration::from_millis(self.next_poll_timeout as u64);

        select! {
            e = self.channel.rx.recv() => {
                self.next_poll_timeout = FRAME_DELAY_MS;
                if let Some(e) = e {
                    if should_handle_event(&e) {
                        return Some(e);
                    }
                }
            }
            _ = tokio::time::sleep(timeout) => {
                self.next_poll_timeout = self.suitable_event_timeout();
            }
        }
        None
    }

    #[inline]
    fn suitable_event_timeout(&self) -> u16 {
        match self.state.visualizer.0 {
            Some(_) => LOW_EVENT_TIMEOUT,
            None => HIGH_EVENT_TIMEOUT,
        }
    }
}

fn chain_hook() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        reset_terminal().unwrap();
        original_hook(panic);
        std::process::exit(1);
    }));
}

pub fn setup_terminal() -> Result<()> {
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    Ok(())
}

pub fn reset_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn should_handle_event(event: &Event) -> bool {
    match event {
        Event::Terminal(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Release => {
            // WARN: we ignore every key release event for now because of a crossterm 0.26
            // quirk: https://github.com/crossterm-rs/crossterm/pull/745
            false
        }
        _ => true,
    }
}
