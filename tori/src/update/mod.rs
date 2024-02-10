use crate::{
    error::Result,
    events::{channel::Tx, Action, Command},
    player::Player,
    state::{browse_screen::Focus, Screen, State}, config::Config,
};
use crossterm::event::{Event as TermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub fn handle_event(state: &mut State<'_>, ev: TermEvent) -> Result<Option<Action>> {
    Ok(match ev {
        TermEvent::Key(key) if key.kind != KeyEventKind::Release => match &state.screen {
            Screen::BrowseScreen(screen) => match &screen.focus {
                Focus::Playlists | Focus::Songs => Some(transform_normal_mode_key(key)),
                Focus::PlaylistsFilter(_) | Focus::SongsFilter(_) => Some(Action::Input(key)),
            },
        },
        _ => None,
    })
}

/// Transforms a key event into the corresponding action, if there is one.
/// Assumes state is in normal mode
fn transform_normal_mode_key(key_event: KeyEvent) -> Action {
    match Config::global().keybindings.get_from_event(key_event) {
        Some(cmd) if cmd != Command::Nop => Action::Command(cmd),
        _ => Action::Input(key_event),
    }
}

pub fn update(state: &mut State<'_>, tx: Tx, act: Action) -> Result<Option<Action>> {
    use Action::*;
    match act {
        Input(_) => {}

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

        SongAdded {
            playlist,
            song: _song,
        } => {
            let Screen::BrowseScreen(screen) = &mut state.screen;
            if screen.selected_playlist() == Some(playlist.as_str()) {
                screen.refresh_songs()?;
                screen
                    .shown_songs
                    .select(Some(screen.shown_songs.items.len() - 1));
            }
        }
        RefreshSongs => {
            let Screen::BrowseScreen(screen) = &mut state.screen;
            screen.refresh_songs()?;
        }
        SelectSong(i) => {
            let Screen::BrowseScreen(screen) = &mut state.screen;
            screen.shown_songs.select(Some(i));
        }
        SelectPlaylist(i) => {
            let Screen::BrowseScreen(screen) = &mut state.screen;
            screen.shown_playlists.select(Some(i));
            screen.refresh_songs()?;
        }

        Command(cmd) => return handle_command(state, tx, cmd),
    }

    Ok(None)
}

fn handle_command(state: &mut State<'_>, _tx: Tx, cmd: Command) -> Result<Option<Action>> {
    use Command::*;
    match cmd {
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
        SelectNext => match &mut state.screen {
            Screen::BrowseScreen(screen) => screen.select_next()?,
        },
        SelectPrev => match &mut state.screen {
            Screen::BrowseScreen(screen) => screen.select_prev()?,
        },
        SelectLeft => match &mut state.screen {
            Screen::BrowseScreen(screen) => screen.focus = Focus::Playlists,
        },
        SelectRight => match &mut state.screen {
            Screen::BrowseScreen(screen) => screen.focus = Focus::Songs,
        },
        Add => todo!(),
        QueueSong => todo!(),
        QueueShown => todo!(),
        PlayFromModal => todo!(),
        OpenInEditor => todo!(),
        Search => todo!(),
    }
    Ok(None)
}

fn push_key_to_filter(filter: &mut String, key: KeyEvent) {
    match key.code {
        KeyCode::Backspace if key.modifiers & KeyModifiers::ALT != KeyModifiers::NONE => {
            // Remove trailing whitespace
            while let Some(c) = filter.pop() {
                if !c.is_whitespace() {
                    break;
                }
            }
            // Remove word
            while let Some(c) = filter.pop() {
                if c.is_whitespace() {
                    filter.push(c);
                    break;
                }
            }
        }
        KeyCode::Backspace => {
            filter.pop();
        }
        KeyCode::Char(c) => {
            filter.push(c);
        }
        _ => {}
    }
}
