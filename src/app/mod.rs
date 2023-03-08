use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use libmpv::Mpv;
use std::{cell::RefCell, error::Error, rc::Rc};
use std::{io, time::Duration};
use tui::{backend::CrosstermBackend, Frame, Terminal};

pub mod browse_screen;

pub trait Screen {
    fn render(&mut self, frame: &mut Frame<'_, MyBackend>);
    fn handle_event(&mut self, app: &mut App, event: Event) -> Result<(), Box<dyn Error>>;
}

pub(crate) type MyBackend = CrosstermBackend<io::Stdout>;

pub struct App {
    terminal: Terminal<MyBackend>,
    mpv: Mpv,
    state: Option<Rc<RefCell<dyn Screen>>>,
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

        Ok(App {
            terminal,
            mpv,
            state: Some(Rc::new(RefCell::new(state))),
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.chain_hook();
        setup_terminal()?;

        while let Some(state_rc) = &self.state {
            let state_rc = state_rc.clone();
            let mut state = state_rc.borrow_mut();
            self.terminal.draw(|f| state.render(f))?;

            if event::poll(Duration::from_millis(1000))? {
                state.handle_event(self, event::read()?)?;
            }
        }

        reset_terminal()?;
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
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}
