use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEventKind,
        KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{borrow::Cow, cell::RefCell, rc::Rc};
use std::{
    io,
    time::{self, Duration},
};
use tokio::select;
use tui::{backend::CrosstermBackend, layout::Rect, style::Color, Terminal};

use crate::{
    component::Visualizer,
    error::Result,
    events::{transform_normal_mode_key, Channel},
    player::{DefaultPlayer, Player},
    widgets::notification::Notification, app::component::Mode,
};

pub mod app_screen;
pub mod browse_screen;
pub mod component;
pub mod filtered_list;
pub mod modal;
pub mod playlist_screen;

use crate::events::Event;

use self::{app_screen::AppScreen, component::Component};

type MyBackend = CrosstermBackend<io::Stdout>;

const FRAME_DELAY_MS: u16 = 16;
const HIGH_EVENT_TIMEOUT: u16 = 1000;
const LOW_EVENT_TIMEOUT: u16 = 17;

pub struct App<'a> {
    pub channel: Channel,
    terminal: Terminal<MyBackend>,
    player: DefaultPlayer,
    next_render: time::Instant,
    next_poll_timeout: u16,
    notification: Notification<'a>,
    visualizer: Visualizer,
    screen: Rc<RefCell<AppScreen<'a>>>,
    quit: bool,
}

impl<'a> App<'a> {
    pub fn new() -> Result<App<'a>> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let player = DefaultPlayer::new()?;

        let screen = Rc::new(RefCell::new(AppScreen::new()?));

        let channel = Channel::new();

        let next_render = time::Instant::now();
        let next_poll_timeout = LOW_EVENT_TIMEOUT;

        let visualizer = Visualizer::default();

        let notification = Notification::default();

        Ok(App {
            channel,
            terminal,
            player,
            next_render,
            next_poll_timeout,
            notification,
            visualizer,
            screen,
            quit: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        chain_hook();
        setup_terminal()?;

        while !self.quit {
            self.render()
                .map_err(|e| self.notify_err(e.to_string()))
                .ok();

            if let Some(event) = self.recv_event().await {
                let screen = self.screen.clone();
                screen
                    .borrow_mut()
                    .handle_event(self, event)
                    .map_err(|e| self.notify_err(e.to_string()))
                    .ok();
            }
        }

        reset_terminal()?;
        Ok(())
    }

    #[inline]
    fn render(&mut self) -> Result<()> {
        if time::Instant::now() >= self.next_render {
            if let Err(err) = self.visualizer.update() {
                self.notify_err(err.to_string());
            }

            self.terminal.draw(|frame| {
                let chunk = frame.size();
                self.screen.borrow_mut().render(frame, chunk, ());
                self.notification.render(frame, chunk, ());
                self.visualizer.render(chunk, frame.buffer_mut());
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
                    let e = self.transform_event(e);
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
        match self.visualizer.0 {
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
                    _ => transform_normal_mode_key(key_event),
                }
            }
            _ => event,
        }
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
