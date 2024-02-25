pub mod browse_screen;
use browse_screen::browse_screen_action;
use tokio::task;

use std::{mem, time::Duration};

use crate::{
    app::modal::{HelpModal, HotkeyModal, InputModal, Modal},
    config::Config,
    error::Result,
    events::{action::Level, channel::Tx, Action, Command},
    input::InputResponse,
    m3u::playlist_management,
    player::Player,
    rect_ops::RectOps,
    state::{browse_screen::Focus, Screen, State},
    ui::UIEvent,
};
use crossterm::event::{Event as TermEvent, KeyEvent, KeyEventKind, MouseEventKind};

pub fn handle_event(state: &mut State<'_>, tx: Tx, ev: TermEvent) -> Result<Option<Action>> {
    if let Some(modal) = &mut state.modal {
        return modal.handle_event(tx, ev);
    }

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
            MouseEventKind::Down(_) => {
                for listener in &state.listeners {
                    if let UIEvent::Click(rect) = listener.event {
                        if rect.contains(mouse.column, mouse.row) {
                            let ev = ev.clone();
                            tx.send((listener.emitter)(ev))?;
                        }
                    }
                }
                None
            }
            MouseEventKind::Drag(_) => {
                for listener in &state.listeners {
                    if let UIEvent::Drag(rect) = listener.event {
                        if rect.contains(mouse.column, mouse.row) {
                            let ev = ev.clone();
                            tx.send((listener.emitter)(ev))?;
                        }
                    }
                }
                None
            }
            MouseEventKind::Up(_) => None,
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
        | SelectAndMaybePlaySong(_)
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

        Notify(level, msg) => {
            match level {
                Level::Ok => state.notify_ok(msg),
                Level::Info => state.notify_info(msg),
                Level::Error => state.notify_err(msg),
            };
        }

        Play(song) => {
            state.player.play(&song)?;
        }

        CloseModal => {
            state.modal = None;
        }
        AddPlaylist { name } => {
            match playlist_management::create_playlist(&name) {
                Ok(()) => state.notify_ok(format!("Playlist {name} created")),
                Err(e) => state.notify_err(format!("Error creating playlist: {e:?}")),
            };
            return Ok(Some(Action::RefreshPlaylists));
        }
        RenamePlaylist { playlist, new_name } => {
            match playlist_management::rename_playlist(&playlist, &new_name) {
                Ok(()) => state.notify_ok(format!("Playlist {playlist} renamed to {new_name}")),
                Err(e) => state.notify_err(format!("Error renaming playlist: {e:?}")),
            }
            return Ok(Some(Action::RefreshPlaylists));
        }
        DeletePlaylist { playlist } => {
            match playlist_management::delete_playlist(&playlist) {
                Ok(()) => state.notify_ok(format!("Playlist {playlist} deleted")),
                Err(e) => state.notify_err(format!("Error deleting playlist: {e:?}")),
            };
            return Ok(Some(Action::RefreshPlaylists));
        }
        AddSongToPlaylist { playlist, song } => {
            tokio::task::spawn(async move {
                let res =
                    playlist_management::add_song(tx.clone(), playlist.clone(), song.clone()).await;
                match res {
                    Ok(()) => tx
                        .send(Action::Notify(
                            Level::Info,
                            format!("Added {song} to {playlist} playlist"),
                        ))
                        .ok(),
                    Err(e) => tx.send(Notify(Level::Error, format!("{:?}", e))).ok(),
                };
                tx.send(RefreshSongs).ok();
            });
        }
        RenameSong {
            playlist,
            index,
            new_name,
        } => {
            match playlist_management::rename_song(&playlist, index, &new_name) {
                Ok(()) => {}
                Err(e) => state.notify_err(format!("Error renaming song: {e:?}")),
            };
            return Ok(Some(Action::RefreshSongs));
        }
        DeleteSong { playlist, index } => {
            match playlist_management::delete_song(&playlist, index) {
                Ok(()) => {}
                Err(e) => state.notify_err(format!("Error deleting song: {e:?}")),
            }
            return Ok(Some(Action::RefreshSongs));
        }

        SetVolume(p) => {
            state.player.set_volume((100. * p).round() as i64)?;
            task::spawn(async move {
                // Wait for the player to actually set the volume
                tokio::time::sleep(Duration::from_millis(100)).await;
                tx.send(Action::Tick).ok();
            });
        }
        SeekAbsolute(p) => {
            state.player.seek_absolute((100. * p).round() as usize)?;
            task::spawn(async move {
                // Wait for the player to update the playback position
                tokio::time::sleep(Duration::from_millis(100)).await;
                tx.send(Action::Tick).ok();
            });
        }
    }

    Ok(None)
}

fn handle_command(state: &mut State<'_>, tx: Tx, cmd: Command) -> Result<Option<Action>> {
    use Command::*;
    match cmd {
        Esc | Play | QueueSong | QueueShown | OpenInBrowser | CopyUrl | CopyTitle
        | NextSortingMode | SelectLeft | SelectNext | SelectRight | SelectPrev | Search
        | GotoStart | GotoEnd | Add | Rename | Delete | SwapSongDown | SwapSongUp => {
            return screen_action(state, tx, Action::Command(cmd))
        }

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

        OpenHelpModal => {
            state.modal = HelpModal::new().some_box();
        }
        OpenHotkeyModal => {
            state.modal = HotkeyModal::default().some_box();
        }
        PlayFromModal => {
            state.modal = InputModal::new(" Play ").on_commit(Action::Play).some_box();
        }

        OpenInEditor => todo!("OpenInEditor is not yet implemented"),
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
