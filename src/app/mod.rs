use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use libmpv::Mpv;
use std::{borrow::Cow, cell::RefCell, rc::Rc, sync::mpsc};
use std::{
    io,
    time::{self, Duration},
};
use tui::{backend::CrosstermBackend, layout::Rect, style::Color, Terminal};

use crate::{
    app::component::Mode,
    command,
    config::Config,
    error::Result,
    events::{self, Channel},
    visualizer::Visualizer,
    widgets::notification::Notification,
};

pub mod app_screen;
pub mod browse_screen;
pub mod component;
pub mod filtered_list;
pub mod modal;
pub mod playlist_screen;

use crate::events::Event;

use self::{
    app_screen::AppScreen,
    component::{Component, MouseHandler, MyBackend},
};

const FRAME_DELAY_MS: u16 = 16;
const HIGH_EVENT_TIMEOUT: u16 = 1000;
const LOW_EVENT_TIMEOUT: u16 = 17;

pub struct App<'a> {
    pub channel: Channel,
    terminal: Terminal<MyBackend>,
    mpv: Mpv,
    next_render: time::Instant,
    next_poll_timeout: u16,
    notification: Notification<'a>,
    visualizer: Option<Visualizer>,
    screen: Rc<RefCell<AppScreen<'a>>>,
    quit: bool,
}

impl<'a> App<'a> {
    pub fn new() -> Result<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let mpv = Mpv::with_initializer(|mpv| {
            mpv.set_property("video", false)
                .and_then(|_| mpv.set_property("volume", 100))
        })?;

        let screen = Rc::new(RefCell::new(AppScreen::new()?));

        let channel = Channel::default();

        let next_render = time::Instant::now();
        let next_poll_timeout = LOW_EVENT_TIMEOUT;

        let notification = Notification::default();

        Ok(App {
            channel,
            terminal,
            mpv,
            next_render,
            next_poll_timeout,
            notification,
            visualizer: None,
            screen,
            quit: false,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.chain_hook();
        setup_terminal()?;

        self.channel.spawn_terminal_event_getter();
        self.channel.spawn_ticks();

        while !self.quit {
            self.render()
                .map_err(|e| self.notify_err(e.to_string()))
                .ok();
            self.recv_event()
                .map_err(|e| self.notify_err(e.to_string()))
                .ok();
        }

        reset_terminal()?;
        Ok(())
    }

    #[inline]
    fn render(&mut self) -> Result<()> {
        if time::Instant::now() >= self.next_render {
            self.terminal.draw(|frame| {
                let chunk = frame.size();
                self.screen.borrow_mut().render(frame, chunk, ());
                self.notification.render(frame, frame.size(), ());
            })?;

            if let Some(ref visualizer) = self.visualizer {
                visualizer.render(self.terminal.current_buffer_mut());
            }

            self.next_render = time::Instant::now()
                .checked_add(Duration::from_millis(FRAME_DELAY_MS as u64))
                .unwrap();
        }
        Ok(())
    }

    #[inline]
    fn recv_event(&mut self) -> Result<()> {
        // NOTE: Big timeout if the last event was long ago, small timeout otherwise.
        // This makes it so after a burst of events, like a Ctrl+V, we get a small timeout
        // immediately after the last event, which triggers a fast render.
        let timeout = Duration::from_millis(self.next_poll_timeout as u64);
        match self.channel.receiver.recv_timeout(timeout) {
            Ok(Event::Terminal(CrosstermEvent::Key(key))) if key.kind == KeyEventKind::Release => {
                // WARN: we ignore every key release event for now because of a crossterm 0.26
                // quirk: https://github.com/crossterm-rs/crossterm/pull/745
                self.next_poll_timeout = FRAME_DELAY_MS;
            }
            Ok(event) => {
                let event = self.transform_event(event);
                self.handle_event(event)?;
                self.next_poll_timeout = FRAME_DELAY_MS;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                self.next_poll_timeout = self.suitable_event_timeout();
            }
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }

    #[inline]
    fn suitable_event_timeout(&self) -> u16 {
        match self.visualizer {
            Some(_) => LOW_EVENT_TIMEOUT,
            None => HIGH_EVENT_TIMEOUT,
        }
    }

    /// Transforms an event, according to the current app state.
    fn transform_event(&self, event: Event) -> Event {
        use Event::*;
        match event {
            Terminal(CrosstermEvent::Key(key_event)) => {
                let has_mods = key_event.modifiers & (KeyModifiers::CONTROL | KeyModifiers::ALT)
                    != KeyModifiers::NONE;
                match self.screen.borrow().mode() {
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

    fn handle_event(&mut self, event: events::Event) -> Result<()> {
        match &event {
            Event::Command(command::Command::ToggleVisualizer) => {
                self.toggle_visualizer()?;
            }
            Event::Terminal(crossterm::event::Event::Mouse(mouse_event)) => {
                let screen = self.screen.clone();
                let chunk = self.frame_size();
                screen
                    .borrow_mut()
                    .handle_mouse(self, chunk, *mouse_event)?;
            }
            _otherwise => {
                let screen = self.screen.clone();
                screen.borrow_mut().handle_event(self, event)?;
            }
        }
        Ok(())
    }

    /// Transforms a key event into the corresponding command, if there is one.
    /// Assumes state is in normal mode
    fn transform_normal_mode_key(&self, key_event: KeyEvent) -> Event {
        use crate::command::Command::Nop;
        use crossterm::event::Event::Key;
        use Event::*;
        match Config::global().keybindings.get_from_event(key_event) {
            Some(cmd) if cmd != Nop => Command(cmd),
            _ => Terminal(Key(key_event)),
        }
    }

    fn toggle_visualizer(&mut self) -> Result<()> {
        if self.visualizer.take().is_none() {
            let opts = crate::visualizer::CavaOptions {
                bars: self.terminal.get_frame().size().width as usize / 2,
            };
            self.visualizer = Some(Visualizer::new(opts)?);
        }
        Ok(())
    }

    fn chain_hook(&mut self) {
        let original_hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |panic| {
            reset_terminal().unwrap();
            original_hook(panic);
            std::process::exit(1);
        }));
    }

    pub fn select_screen(&mut self, screen: app_screen::Selected) {
        self.screen.borrow_mut().select(screen);
    }

    pub fn quit(&mut self) {
        self.quit = true;
    }

    ////////////////////////////////
    //        Notification        //
    ////////////////////////////////
    pub fn notify_err(&mut self, err: impl Into<Cow<'a, str>>) {
        self.notification = Notification::new(err, Duration::from_secs(5)).colored(Color::LightRed);
    }

    pub fn notify_info(&mut self, info: impl Into<Cow<'a, str>>) {
        self.notification =
            Notification::new(info, Duration::from_secs(4)).colored(Color::LightCyan);
    }

    pub fn notify_ok(&mut self, text: impl Into<Cow<'a, str>>) {
        self.notification =
            Notification::new(text, Duration::from_secs(4)).colored(Color::LightGreen);
    }

    /////////////////////////
    //        Frame        //
    /////////////////////////
    pub fn frame_size(&mut self) -> Rect {
        self.terminal.get_frame().size()
    }
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
