use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, sync::Arc};
use tokio::{
    select,
    sync::{mpsc, Mutex},
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::{
    error::Result,
    events::channel::Channel,
    state::State,
    ui::ui,
    update::{handle_event, update},
};

pub mod filtered_list;
pub mod modal;

type MyBackend = CrosstermBackend<io::Stdout>;

/// Controls the application main loop
pub struct App<'n> {
    pub state: Arc<Mutex<State<'n>>>,
    pub channel: Channel,
    pub terminal: Terminal<MyBackend>,
}

impl<'a> App<'a> {
    pub fn new() -> Result<App<'a>> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);

        Ok(App {
            state: Arc::new(Mutex::new(State::new()?)),
            channel: Channel::new(),
            terminal: Terminal::new(backend)?,
        })
    }

    pub async fn run(self) -> Result<()> {
        chain_hook();
        setup_terminal()?;

        tokio_scoped::scope(|scope| {
            let App {
                state: state_,
                mut channel,
                mut terminal,
            } = self;

            let (render_tx_, mut render_rx) = mpsc::channel::<()>(1);

            // Updating task
            let state = state_.clone();
            let render_tx = render_tx_.clone();
            scope.spawn(async move {
                while !state.lock().await.quit {
                    App::recv(&state, &mut channel).await;
                    render_tx
                        .send(())
                        .await
                        .expect("Failed to send render event");
                }
            });

            // Rendering task
            let state = state_.clone();
            scope.spawn(async move {
                while !state.lock().await.quit {
                    if render_rx.recv().await.is_some() {
                        let mut state = state.lock().await;
                        App::render(&mut terminal, &mut state)
                            .map_err(|e| state.notify_err(format!("Rendering error: {e}")))
                            .ok();
                    }
                }
            });

            // Trigger first render immediately
            let render_tx = render_tx_.clone();
            scope.spawn(async move {
                render_tx
                    .send(())
                    .await
                    .expect("Failed to send render event");
            });
        });

        reset_terminal()?;
        Ok(())
    }

    async fn recv(state: &Mutex<State<'_>>, channel: &mut Channel) {
        select! {
            crossterm_event = channel.crossterm_rx.recv() => {
                if let Some(ev) = crossterm_event {
                    let mut state = state.lock().await;
                    match handle_event(&mut state, ev) {
                        Ok(Some(a)) => channel.tx.send(a).expect("Failed to send action"),
                        Ok(None) => {}
                        Err(e) => state.notify_err(e.to_string()),
                    }
                }
            }

            action = channel.rx.recv() => {
                if let Some(ev) = action {
                    let mut state = state.lock().await;
                    match update(&mut state, channel.tx.clone(), ev) {
                        Ok(Some(a)) => channel.tx.send(a).expect("Failed to send action"),
                        Ok(None) => {}
                        Err(e) => state.notify_err(e.to_string()),
                    }
                }
            }
        }
    }

    fn render(terminal: &mut Terminal<MyBackend>, state: &mut State<'_>) -> Result<()> {
        terminal.draw(|frame| {
            let area = frame.size();
            let buf = frame.buffer_mut();
            ui(state, area, buf);
        })?;

        Ok(())
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
