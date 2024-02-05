use std::mem;

use crate::{
    error::Result,
    events::{self, channel::Tx, transform_normal_mode_key, Action, Command, Event},
    player::Player,
    state::{browse_screen::Focus, Screen, State},
};
use crossterm::event::{Event as TermEvent, KeyCode, KeyEvent, KeyModifiers};

pub fn update(state: &mut State<'_>, tx: Tx, ev: Event) -> Result<Option<Event>> {
    use events::Event::*;

    // TODO: return an action instead
    let inner_event = match ev {
        Terminal(TermEvent::Key(key)) => match &mut state.screen {
            Screen::BrowseScreen(screen) => {
                let focus = mem::take(&mut screen.focus);
                match focus {
                    Focus::Playlists | Focus::Songs => {
                        let event = transform_normal_mode_key(key);
                        screen.focus = focus;
                        event
                    }
                    Focus::PlaylistsFilter(mut f) => {
                        push_key_to_filter(&mut f, key);
                        screen.focus = Focus::PlaylistsFilter(f);
                        return Ok(None);
                    }
                    Focus::SongsFilter(mut f) => {
                        push_key_to_filter(&mut f, key);
                        screen.focus = Focus::SongsFilter(f);
                        return Ok(None);
                    }
                }
            }
        },

        otherwise => otherwise,
    };

    update_inner(state, tx, inner_event)
}

// TODO: separate events from actions
pub fn update_inner(state: &mut State<'_>, tx: Tx, ev: Event) -> Result<Option<Event>> {
    use events::Event::*;

    match ev {
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

        Terminal(TermEvent::Key(_key)) => {}
        Terminal(TermEvent::Mouse(_mouse)) => {}
        Terminal(_) => {}

        Action(act) => return handle_action(state, tx, act),
        Command(cmd) => return handle_command(state, tx, cmd),
    }

    Ok(None)
}

fn handle_action(state: &mut State<'_>, _tx: Tx, act: Action) -> Result<Option<Event>> {
    use Action::*;
    match act {
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
    }
    Ok(None)
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
