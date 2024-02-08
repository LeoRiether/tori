use std::mem;

use crate::{
    config::Config,
    error::Result,
    events::{channel::Tx, Action, Command},
    input::Input,
    player::Player,
    state::{browse_screen::Focus, Screen, State},
    util::copy_to_clipboard,
};
use crossterm::event::{Event as TermEvent, KeyCode, KeyEvent, KeyEventKind};

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
        Rerender => {
            // just triggered a rerender
        }

        Input(key) => match &mut state.screen {
            Screen::BrowseScreen(screen) => match &mut screen.focus {
                Focus::PlaylistsFilter(filter) | Focus::SongsFilter(filter) => {
                    match key.code {
                        KeyCode::Esc => {
                            let focus = mem::take(&mut screen.focus);
                            screen.focus = match focus {
                                Focus::PlaylistsFilter(_) => Focus::Playlists,
                                Focus::SongsFilter(_) => Focus::Songs,
                                _ => focus,
                            };
                        }
                        _ => filter.handle_event(key),
                    }
                    screen.refresh_playlists()?;
                }
                _ => {}
            },
        },

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

        Play => match &state.screen {
            Screen::BrowseScreen(screen) => {
                if let Some(song) = screen.selected_song() {
                    state.player.play(&song.path)?;
                }
            }
        },
        QueueSong => match &state.screen {
            Screen::BrowseScreen(screen) => {
                if let Some(song) = screen.selected_song() {
                    state.player.queue(&song.path)?;
                }
            }
        },
        QueueShown => match &state.screen {
            Screen::BrowseScreen(screen) => {
                for &i in screen.shown_songs.iter() {
                    let path = screen.songs[i].path.as_str();
                    state.player.queue(path)?;
                }
            }
        },
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

        OpenInBrowser => match &state.screen {
            Screen::BrowseScreen(screen) => {
                if let Some(song) = screen.selected_song() {
                    match webbrowser::open(&song.path) {
                        Ok(_) => state.notify_ok(format!("Opening {} in your browser", song.path)),
                        Err(e) => state.notify_err(format!("Failed to open song path: {}", e)),
                    }
                }
            }
        },
        CopyUrl => match &state.screen {
            Screen::BrowseScreen(screen) => {
                if let Some(song) = screen.selected_song() {
                    copy_to_clipboard(song.path.clone());
                    state.notify_ok(format!("Copied {} to the clipboard", song.path));
                }
            }
        },
        CopyTitle => match &state.screen {
            Screen::BrowseScreen(screen) => {
                if let Some(song) = screen.selected_song() {
                    copy_to_clipboard(song.title.clone());
                    state.notify_ok(format!("Copied {} to the clipboard", song.title));
                }
            }
        },

        ToggleVisualizer => state.visualizer.toggle()?,

        NextSortingMode => match &mut state.screen {
            Screen::BrowseScreen(screen) => screen.next_sorting_mode(),
        },

        OpenHelpModal => todo!(),
        OpenHotkeyModal => todo!(),
        Add => todo!(),
        Rename => todo!(),
        Delete => todo!(),

        SwapSongDown => todo!(),
        SwapSongUp => todo!(),

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

        PlayFromModal => todo!(),
        OpenInEditor => todo!(),
        Search => match &mut state.screen {
            Screen::BrowseScreen(screen) => {
                let focus = mem::take(&mut screen.focus);
                let new_focus = match focus {
                    Focus::Playlists => Focus::PlaylistsFilter(Input::default()),
                    Focus::Songs => Focus::SongsFilter(Input::default()),
                    _ => focus,
                };
                screen.focus = new_focus;
            }
        },

        GotoStart => match &mut state.screen {
            Screen::BrowseScreen(screen) => match &screen.focus {
                Focus::Playlists => screen.shown_playlists.select_first(),
                Focus::Songs => screen.shown_songs.select_first(),
                Focus::PlaylistsFilter(_) | Focus::SongsFilter(_) => {}
            },
        },
        GotoEnd => match &mut state.screen {
            Screen::BrowseScreen(screen) => match &screen.focus {
                Focus::Playlists => screen.shown_playlists.select_last(),
                Focus::Songs => screen.shown_songs.select_last(),
                Focus::PlaylistsFilter(_) | Focus::SongsFilter(_) => {}
            },
        },
    }

    Ok(None)
}
