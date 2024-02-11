use std::mem;

use tui::style::{Color, Style};

use crate::{
    app::modal::{ConfirmationModal, InputModal, Modal},
    error::Result,
    events::{channel::Tx, Action, Command},
    input::Input,
    m3u::playlist_management,
    player::Player,
    state::{
        browse_screen::{BrowseScreen, Focus},
        State,
    },
    util::copy_to_clipboard,
};

pub fn browse_screen_action(
    state: &mut State<'_>,
    screen: &mut BrowseScreen,
    tx: Tx,
    act: Action,
) -> Result<Option<Action>> {
    use Action::*;
    match act {
        Action::Command(cmd) => return browse_screen_command(state, screen, tx, cmd),

        ScrollDown => match &screen.focus {
            Focus::Playlists => {
                screen.shown_playlists.select_next();
                screen.refresh_songs()?;
            }
            Focus::Songs => screen.shown_songs.select_next(),
            Focus::PlaylistsFilter(_) | Focus::SongsFilter(_) => {}
        },
        ScrollUp => match &screen.focus {
            Focus::Playlists => {
                screen.shown_playlists.select_prev();
                screen.refresh_songs()?;
            }
            Focus::Songs => screen.shown_songs.select_prev(),
            Focus::PlaylistsFilter(_) | Focus::SongsFilter(_) => {}
        },

        SongAdded {
            playlist,
            song: _song,
        } => {
            if screen.selected_playlist() == Some(playlist.as_str()) {
                screen.refresh_songs()?;
                screen
                    .shown_songs
                    .select(Some(screen.shown_songs.items.len() - 1));
            }
        }
        RefreshSongs => screen.refresh_songs()?,
        RefreshPlaylists => screen.refresh_playlists()?,
        SelectAndMaybePlaySong(i) => {
            if screen.shown_songs.selected_item() == Some(i) {
                // Double click => Play song
                if let Some(song) = screen.selected_song() {
                    state.player.play(&song.path)?;
                }
            } else {
                // Select i-th song
                return Ok(Some(Action::SelectSong(i)));
            }
        }
        SelectSong(i) => {
            screen.shown_songs.select(Some(i));
            if let Focus::Playlists | Focus::PlaylistsFilter(_) = screen.focus {
                screen.focus = Focus::Songs;
            }
        }
        SelectPlaylist(i) => {
            screen.shown_playlists.select(Some(i));
            screen.refresh_songs()?;
            if let Focus::Songs | Focus::SongsFilter(_) = screen.focus {
                screen.focus = Focus::Playlists;
            }
        }

        _ => {}
    }

    Ok(None)
}

fn browse_screen_command(
    state: &mut State<'_>,
    screen: &mut BrowseScreen,
    _tx: Tx,
    cmd: Command,
) -> Result<Option<Action>> {
    use Command::*;
    match cmd {
        Esc => {
            let focus = mem::take(&mut screen.focus);
            screen.focus = match focus {
                Focus::PlaylistsFilter(_) => Focus::Playlists,
                Focus::SongsFilter(_) => Focus::Songs,
                _ => focus,
            };
            return Ok(Some(Action::RefreshPlaylists));
        }

        Play => {
            if let Some(song) = screen.selected_song() {
                state.player.play(&song.path)?;
            }
        }
        QueueSong => {
            if let Some(song) = screen.selected_song() {
                state.player.queue(&song.path)?;
            }
        }
        QueueShown => {
            for &i in screen.shown_songs.iter() {
                let path = screen.songs[i].path.as_str();
                state.player.queue(path)?;
            }
        }

        OpenInBrowser => {
            if let Some(song) = screen.selected_song() {
                match webbrowser::open(&song.path) {
                    Ok(_) => state.notify_ok(format!("Opening {} in your browser", song.path)),
                    Err(e) => state.notify_err(format!("Failed to open song path: {}", e)),
                }
            }
        }
        CopyUrl => {
            if let Some(song) = screen.selected_song() {
                copy_to_clipboard(song.path.clone());
                state.notify_ok(format!("Copied {} to the clipboard", song.path));
            }
        }
        CopyTitle => {
            if let Some(song) = screen.selected_song() {
                copy_to_clipboard(song.title.clone());
                state.notify_ok(format!("Copied {} to the clipboard", song.title));
            }
        }

        NextSortingMode => screen.next_sorting_mode(),

        SelectNext => screen.select_next()?,
        SelectPrev => screen.select_prev()?,
        SelectLeft => screen.focus = Focus::Playlists,
        SelectRight => screen.focus = Focus::Songs,

        Search => {
            let focus = mem::take(&mut screen.focus);
            let new_focus = match focus {
                Focus::Playlists => Focus::PlaylistsFilter(Input::default()),
                Focus::Songs => Focus::SongsFilter(Input::default()),
                _ => focus,
            };
            screen.focus = new_focus;
        }

        GotoStart => match &screen.focus {
            Focus::Playlists => screen.shown_playlists.select_first(),
            Focus::Songs => screen.shown_songs.select_first(),
            Focus::PlaylistsFilter(_) | Focus::SongsFilter(_) => {}
        },
        GotoEnd => match &screen.focus {
            Focus::Playlists => screen.shown_playlists.select_last(),
            Focus::Songs => screen.shown_songs.select_last(),
            Focus::PlaylistsFilter(_) | Focus::SongsFilter(_) => {}
        },

        Add => match &screen.focus {
            Focus::Playlists | Focus::PlaylistsFilter(_) => {
                state.modal = InputModal::new(" Add playlist ")
                    .style(Style::default().fg(Color::LightBlue))
                    .on_commit(|name| Action::AddPlaylist { name })
                    .some_box();
            }
            Focus::Songs | Focus::SongsFilter(_) => {
                let playlist = match screen.selected_playlist() {
                    None => {
                        state.notify_err("Can't add a song without a playlist selected");
                        return Ok(None);
                    }
                    Some(p) => p.to_string(),
                };

                state.modal = InputModal::new(" Add song ")
                    .style(Style::default().fg(Color::LightBlue))
                    .on_commit(move |song| Action::AddSongToPlaylist { song, playlist })
                    .some_box();
            }
        },
        Rename => {
            let playlist = match screen.selected_playlist() {
                None => {
                    state.notify_err("Can't rename a song without a playlist selected");
                    return Ok(None);
                }
                Some(p) => p.to_string(),
            };
            match &screen.focus {
                Focus::Playlists | Focus::PlaylistsFilter(_) => {
                    state.modal = InputModal::new(" Rename playlist ")
                        .style(Style::default().fg(Color::LightBlue))
                        .on_commit(move |new_name| Action::RenamePlaylist { playlist, new_name })
                        .some_box();
                }
                Focus::Songs | Focus::SongsFilter(_) => {
                    let index = match screen.shown_songs.selected_item() {
                        None => {
                            state.notify_err("Can't rename a song without one selected");
                            return Ok(None);
                        }
                        Some(i) => i,
                    };

                    state.modal = InputModal::new(" Rename song ")
                        .style(Style::default().fg(Color::LightBlue))
                        .on_commit(move |new_name| Action::RenameSong {
                            playlist: playlist.clone(),
                            index,
                            new_name,
                        })
                        .some_box();
                }
            }
        }
        Delete => {
            let playlist = match screen.selected_playlist() {
                None => {
                    state.notify_err("Can't delete a song without a playlist selected");
                    return Ok(None);
                }
                Some(p) => p.to_string(),
            };
            match &screen.focus {
                Focus::Playlists | Focus::PlaylistsFilter(_) => {
                    state.modal = ConfirmationModal::new(" Delete playlist ")
                        .style(Style::default().fg(Color::LightRed))
                        .on_yes(move || Action::DeletePlaylist { playlist })
                        .some_box();
                }
                Focus::Songs | Focus::SongsFilter(_) => {
                    let index = match screen.shown_songs.selected_item() {
                        None => {
                            state.notify_err("Can't delete a song without one selected");
                            return Ok(None);
                        }
                        Some(i) => i,
                    };

                    state.modal = ConfirmationModal::new(" Delete song ")
                        .style(Style::default().fg(Color::LightRed))
                        .on_yes(move || Action::DeleteSong {
                            playlist: playlist.clone(),
                            index,
                        })
                        .some_box();
                }
            }
        }

        SwapSongUp => {
            if let Some(playlist) = screen.selected_playlist() {
                if let Some(i) = screen.selected_song_index() {
                    if i > 0 {
                        playlist_management::swap_song(playlist, i - 1)?;
                        screen.songs.swap(i - 1, i);
                        screen.shown_songs.select_prev();
                    }
                }
            }
        }
        SwapSongDown => {
            if let Some(playlist) = screen.selected_playlist() {
                if let Some(i) = screen.selected_song_index() {
                    if i + 1 < screen.songs.len() {
                        playlist_management::swap_song(playlist, i)?;
                        screen.songs.swap(i, i + 1);
                        screen.shown_songs.select_next();
                    }
                }
            }
        }

        _ => {}
    }

    Ok(None)
}
