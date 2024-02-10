use crate::{
    error::Result,
    events::{self, channel::Tx, Command, Event, Action},
    player::Player,
    state::State,
};
use crossterm::event::{Event as TermEvent, KeyCode};

pub fn update(state: &mut State<'_>, tx: Tx, ev: Event) -> Result<Option<Event>> {
    use events::Event::*;

    match ev {
        Tick => {
            state.now_playing.update(&state.player);
            state.visualizer.update()?;
            if state.notification.as_ref().filter(|n| n.is_expired()).is_some() {
                state.notification = None;
            }
        }

        Terminal(TermEvent::Key(key)) => {
            if key.code == KeyCode::Char('q') {
                state.quit();
            }
        },
        Terminal(TermEvent::Mouse(_mouse)) => {}
        Terminal(_) => {}

        Action(act) => return handle_action(state, tx, act),
        Command(cmd) => return handle_command(state, tx, cmd),
    }

    Ok(None)
}

fn handle_action(_state: &mut State<'_>, _tx: Tx, _act: Action) -> Result<Option<Event>> {
    todo!()
}

fn handle_command(state: &mut State<'_>, _tx: Tx, cmd: Command) -> Result<Option<Event>> {
    use Command::*;
    match cmd {
        Nop => {}
        Quit => {
            state.quit();
        }
        SeekForward => {
            state.player.seek(10.)?;
            return Ok(Some(Event::Tick));
        }
        SeekBackward => {
            state.player.seek(-10.)?;
            return Ok(Some(Event::Tick));
        }
        NextSong => {
            state
                .player
                .playlist_next()
                .unwrap_or_else(|_| state.notify_err("No next song"));
            return Ok(Some(Event::Tick));
        }
        PrevSong => {
            state
                .player
                .playlist_previous()
                .unwrap_or_else(|_| state.notify_err("No previous song"));
            return Ok(Some(Event::Tick));
        }
        TogglePause => {
            state.player.toggle_pause()?;
            return Ok(Some(Event::Tick));
        }
        ToggleLoop => {
            state.player.toggle_loop_file()?;
            return Ok(Some(Event::Tick));
        }
        VolumeUp => {
            state.player.add_volume(5)?;
            return Ok(Some(Event::Tick));
        }
        VolumeDown => {
            state.player.add_volume(-5)?;
            return Ok(Some(Event::Tick));
        }
        Mute => {
            state.player.toggle_mute()?;
            return Ok(Some(Event::Tick));
        }
        OpenInBrowser => todo!(),
        CopyUrl => todo!(),
        CopyTitle => todo!(),
        ToggleVisualizer => todo!(),
        NextSortingMode => todo!(),
        OpenHelpModal => todo!(),
        OpenHotkeyModal => todo!(),
        Rename => todo!(),
        Delete => todo!(),
        SwapSongDown => todo!(),
        SwapSongUp => todo!(),
        Shuffle => todo!(),
        SelectNext => todo!(),
        SelectPrev => todo!(),
        SelectRight => todo!(),
        SelectLeft => todo!(),
        Add => todo!(),
        QueueSong => todo!(),
        QueueShown => todo!(),
        PlayFromModal => todo!(),
        OpenInEditor => todo!(),
        Search => todo!(),
    }
    Ok(None)
}
