use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use libmpv::Mpv;
use std::{cell::RefCell, error::Error, rc::Rc};
use std::{
    io,
    time::{self, Duration},
};
use tui::{backend::CrosstermBackend, Frame, Terminal};

pub mod browse_screen;
pub mod filtered_list;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(i8)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

pub trait Screen {
    fn render(&mut self, frame: &mut Frame<'_, MyBackend>);
    fn handle_event(&mut self, app: &mut App, event: Event) -> Result<(), Box<dyn Error>>;
}

pub(crate) type MyBackend = CrosstermBackend<io::Stdout>;

pub struct App {
    terminal: Terminal<MyBackend>,
    mpv: Mpv,
    state: Option<Rc<RefCell<dyn Screen>>>,
    next_render: time::Instant,
    next_poll_timeout: u16,
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

        let next_render = time::Instant::now();
        let next_poll_timeout = 1000;

        Ok(App {
            terminal,
            mpv,
            state: Some(Rc::new(RefCell::new(state))),
            next_render,
            next_poll_timeout,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.chain_hook();
        setup_terminal()?;

        while let Some(state_rc) = &self.state {
            let state_rc = state_rc.clone();
            let mut state = state_rc.borrow_mut();

            self.render(&mut *state)?;
            self.handle_event(&mut *state)?;
        }

        reset_terminal()?;
        Ok(())
    }

    #[inline]
    fn render(&mut self, state: &mut dyn Screen) -> Result<(), Box<dyn Error>> {
        if time::Instant::now() >= self.next_render {
            self.terminal.draw(|f| state.render(f))?;
            self.next_render = time::Instant::now()
                .checked_add(Duration::from_millis(8))
                .unwrap();
        }
        Ok(())
    }

    #[inline]
    fn handle_event(&mut self, state: &mut dyn Screen) -> Result<(), Box<dyn Error>> {
        // NOTE: Big timeout if the last event was long ago, small timeout otherwise.
        // This makes it so after a burst of events, like a Ctrl+V, we get a small timeout
        // just after the last event, which triggers a fast render.
        if event::poll(Duration::from_millis(self.next_poll_timeout as u64))? {
            state.handle_event(self, event::read()?)?;
            self.next_poll_timeout = 8;
        } else {
            self.next_poll_timeout = 1000;
        }
        Ok(())
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
