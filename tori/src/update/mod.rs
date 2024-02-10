pub mod browse_screen;
use browse_screen::browse_screen_action;

use std::mem;

use crate::{
    config::Config,
    error::Result,
    events::{channel::Tx, Action, Command},
    input::InputResponse,
    player::Player,
    state::{browse_screen::Focus, Screen, State},
};
use crossterm::event::{Event as TermEvent, KeyEvent, KeyEventKind, MouseEventKind};

pub fn handle_event(state: &mut State<'_>, ev: TermEvent) -> Result<Option<Action>> {
    Ok(match ev {
        TermEvent::Key(key) if key.kind != KeyEventKind::Release => match &mut state.screen {
            Screen::None => unreachable!(),
            Screen::BrowseScreen(screen) => match &mut screen.focus {
                Focus::Playlists | Focus::Songs => transform_normal_mode_key(key),
                Focus::PlaylistsFilter(filter) | Focus::SongsFilter(filter) => {
                    let event_handled = filter.handle_event(key);
                    match event_handled {
                        InputResponse::Handled => Some(Action::RefreshPlaylists),
                        InputResponse::NotHandled => transform_normal_mode_key(key),
                    }
                }
            },
        },
        TermEvent::Mouse(mouse) => match mouse.kind {
            MouseEventKind::Down(_) => todo!("Clicks are yet to be implemented!"),
            MouseEventKind::Up(_) => todo!("Clickups are yet to be implemented!"),
            MouseEventKind::Drag(_) => todo!("Dragging is yet to be implemented!"),
            MouseEventKind::ScrollDown => Some(Action::ScrollDown),
            MouseEventKind::ScrollUp => Some(Action::ScrollUp),
            MouseEventKind::Moved | MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => {
                None
            }
        },
        _ => None,
    })
}

/// Transforms a key event into the corresponding action, if there is one.
/// Assumes state is in normal mode
fn transform_normal_mode_key(key_event: KeyEvent) -> Option<Action> {
    match Config::global().keybindings.get_from_event(key_event) {
        Some(cmd) if cmd != Command::Nop => Some(Action::Command(cmd)),
        _ => None,
    }
}

pub fn update(state: &mut State<'_>, tx: Tx, act: Action) -> Result<Option<Action>> {
    use Action::*;
    match act {
        Command(cmd) => return handle_command(state, tx, cmd),

        ScrollDown
        | ScrollUp
        | SongAdded { .. }
        | RefreshSongs
        | RefreshPlaylists
        | SelectSong(_)
        | SelectPlaylist(_) => return screen_action(state, tx, act),

        Rerender => {
            // just triggered a rerender
        }

        Tick => {
            state.now_playing.update(&state.player);
            state.visualizer.update()?;
            if state
                .notification
                .as_ref()
                .filter(|n| n.is_expired())
                .is_some()
            {
                state.notification = None;
            }
        }

        CloseModal => {
            state.modal = None;
        }
    }

    Ok(None)
}

fn handle_command(state: &mut State<'_>, tx: Tx, cmd: Command) -> Result<Option<Action>> {
    use Command::*;
    match cmd {
        Esc | Play | QueueSong | QueueShown | OpenInBrowser | CopyUrl | CopyTitle
        | NextSortingMode | SelectLeft | SelectNext | SelectRight | SelectPrev | Search
        | GotoStart | GotoEnd => return screen_action(state, tx, Action::Command(cmd)),

        Nop => {}
        Quit => {
            state.quit();
        }

        SeekForward => {
            state.player.seek(10.)?;
            return Ok(Some(Action::Tick));
        }
        SeekBackward => {
            state.player.seek(-10.)?;
            return Ok(Some(Action::Tick));
        }

        NextSong => {
            state
                .player
                .playlist_next()
                .unwrap_or_else(|_| state.notify_err("No next song"));
            return Ok(Some(Action::Tick));
        }
        PrevSong => {
            state
                .player
                .playlist_previous()
                .unwrap_or_else(|_| state.notify_err("No previous song"));
            return Ok(Some(Action::Tick));
        }
        Shuffle => {
            state.player.shuffle()?;
        }

        TogglePause => {
            state.player.toggle_pause()?;
            return Ok(Some(Action::Tick));
        }
        ToggleLoop => {
            state.player.toggle_loop_file()?;
            return Ok(Some(Action::Tick));
        }

        VolumeUp => {
            state.player.add_volume(5)?;
            return Ok(Some(Action::Tick));
        }
        VolumeDown => {
            state.player.add_volume(-5)?;
            return Ok(Some(Action::Tick));
        }
        Mute => {
            state.player.toggle_mute()?;
            return Ok(Some(Action::Tick));
        }

        ToggleVisualizer => state.visualizer.toggle()?,

        OpenHelpModal => todo!(),
        OpenHotkeyModal => todo!(),
        Add => todo!(),
        Rename => todo!(),
        Delete => todo!(),
        PlayFromModal => todo!(),

        SwapSongDown => todo!(),
        SwapSongUp => todo!(),

        OpenInEditor => todo!(),
    }

    Ok(None)
}

fn screen_action(state: &mut State<'_>, tx: Tx, action: Action) -> Result<Option<Action>> {
    let mut screen = mem::take(&mut state.screen);

    let res = match &mut screen {
        Screen::None => Ok(None),
        Screen::BrowseScreen(screen) => browse_screen_action(state, screen, tx, action),
    };

    // If state.screen != None, then the screen action changed the type of screen! We want to keep
    // this change
    if let Screen::None = state.screen {
        state.screen = screen;
    }

    res
}
