use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use libmpv::Mpv;
use notification::Notification;
use std::{cell::RefCell, error::Error, rc::Rc, sync::mpsc};
use std::{
    io,
    time::{self, Duration},
};
use tui::{backend::CrosstermBackend, style::Color, Frame, Terminal};

use crate::{events::{self, Channel}, config::Config};

pub mod browse_screen;
pub mod filtered_list;
pub mod notification;
pub mod modal;

use crate::events::Event;

const FRAME_DELAY_MS: u16 = 16;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(i8)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

pub trait Screen {
    fn mode(&self) -> Mode;
    fn render(&mut self, frame: &mut Frame<'_, MyBackend>);
    fn handle_event(&mut self, app: &mut App, event: events::Event) -> Result<(), Box<dyn Error>>;
}

pub(crate) type MyBackend = CrosstermBackend<io::Stdout>;

pub struct App {
    terminal: Terminal<MyBackend>,
    mpv: Mpv,
    state: Option<Rc<RefCell<dyn Screen>>>,
    channel: Channel,
    next_render: time::Instant,
    next_poll_timeout: u16,
    notification: Notification,
}

impl App {
    pub fn new<S: Screen + 'static>(state: S) -> Result<Self, Box<dyn Error>> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let mpv = Mpv::with_initializer(|mpv| {
            mpv.set_property("video", false)
                .and_then(|_| mpv.set_property("volume", 100))
        })?;

        let channel = Channel::default();

        let next_render = time::Instant::now();
        let next_poll_timeout = 1000;

        let notification = Notification::default();

        Ok(App {
            terminal,
            mpv,
            state: Some(Rc::new(RefCell::new(state))),
            channel,
            next_render,
            next_poll_timeout,
            notification,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.chain_hook();
        setup_terminal()?;

        self.channel.spawn_terminal_event_getter();
        self.channel.spawn_ticks();

        while let Some(state_rc) = &self.state {
            let state_rc = state_rc.clone();
            let mut state = state_rc.borrow_mut();

            self.render(&mut *state)
                .map_err(|e| self.notify_err(e.to_string()))
                .ok();
            self.handle_event(&mut *state)
                .map_err(|e| self.notify_err(e.to_string()))
                .ok();
        }

        reset_terminal()?;
        Ok(())
    }

    #[inline]
    fn render(&mut self, state: &mut dyn Screen) -> Result<(), Box<dyn Error>> {
        if time::Instant::now() >= self.next_render {
            self.terminal.draw(|f| {
                state.render(f);
                self.notification.render(f);
            })?;

            self.next_render = time::Instant::now()
                .checked_add(Duration::from_millis(FRAME_DELAY_MS as u64))
                .unwrap();
        }
        Ok(())
    }

    #[inline]
    fn handle_event(&mut self, state: &mut dyn Screen) -> Result<(), Box<dyn Error>> {
        // NOTE: Big timeout if the last event was long ago, small timeout otherwise.
        // This makes it so after a burst of events, like a Ctrl+V, we get a small timeout
        // immediately after the last event, which triggers a fast render.
        let timeout = Duration::from_millis(self.next_poll_timeout as u64);
        match self.channel.receiver.recv_timeout(timeout) {
            Ok(event) => {
                let event = self.transform_event(state, event);
                state.handle_event(self, event)?;
                self.next_poll_timeout = FRAME_DELAY_MS;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                self.next_poll_timeout = 1000;
            }
            Err(e) => {
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// Transforms an event, according to the current app state.
    fn transform_event(&self, state: &mut dyn Screen, event: Event) -> Event {
        use Event::*;
        match event {
            Terminal(crossterm::event::Event::Key(key_event)) => {
                let has_mods = key_event.modifiers & (KeyModifiers::CONTROL | KeyModifiers::ALT)
                    != KeyModifiers::NONE;
                match state.mode() {
                    // In insert mode, key events pass through untransformed, unless there's a
                    // control or alt modifier
                    Mode::Insert if !has_mods => event,

                    // Otherwise, events may be transformed into commands
                    _ => self.transform_normal_mode_key(key_event),
                }
            }
            _ => event,
        }
    }

    /// Transforms a key event into the corresponding command, if there is one.
    /// Assumes state is in normal mode
    fn transform_normal_mode_key(&self, key_event: KeyEvent) -> Event {
        use crossterm::event::Event::Key;
        use Event::*;
        match Config::global().normal.get_from_event(key_event) {
            Some(cmd) => Command(cmd),
            None => Terminal(Key(key_event)),
        }
    }

    fn chain_hook(&mut self) {
        let original_hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |panic| {
            reset_terminal().unwrap();
            original_hook(panic);
        }));
    }

    pub fn change_state(&mut self, state: Option<Rc<RefCell<dyn Screen>>>) {
        self.state = state;
    }

    ////////////////////////////////
    //        Notification        //
    ////////////////////////////////
    pub fn notify_err(&mut self, err: String) {
        self.notification = Notification::new(err, Duration::from_secs(5)).colored(Color::LightRed);
    }

    pub fn notify_info(&mut self, info: String) {
        self.notification =
            Notification::new(info, Duration::from_secs(4)).colored(Color::LightCyan);
    }

    pub fn notify_ok(&mut self, text: String) {
        self.notification =
            Notification::new(text, Duration::from_secs(4)).colored(Color::LightGreen);
    }
}

pub fn setup_terminal() -> Result<(), Box<dyn Error>> {
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    Ok(())
}

pub fn reset_terminal() -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
