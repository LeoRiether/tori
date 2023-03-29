use crate::app::MyBackend;
use crate::app::Screen;
use crate::App;

use crate::app::playlist_management;
use crate::command;
use crate::events::Event;
use crossterm::event::KeyCode;
use std::error::Error;

use tui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

mod playlists;
use playlists::PlaylistsPane;

mod songs;
use songs::SongsPane;

mod now_playing;
use now_playing::NowPlaying;

use super::modal::{self, Modal};
use super::Mode;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ModalType {
    AddSong { playlist: String },
    AddPlaylist,
    Play,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[repr(i8)]
enum BrowsePane {
    #[default]
    Playlists,
    Songs,
    Modal(ModalType),
}

#[derive(Debug, Default)]
pub struct BrowseScreen {
    playlists: PlaylistsPane,
    songs: SongsPane,
    modal: Modal,
    selected_pane: BrowsePane,

    now_playing: NowPlaying,
}

impl BrowseScreen {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let playlists = PlaylistsPane::new()?;
        let songs = SongsPane::from_playlist_pane(&playlists);
        Ok(Self {
            playlists,
            songs,
            ..Default::default()
        })
    }

    pub fn reload_songs(&mut self) {
        self.songs = SongsPane::from_playlist_pane(&self.playlists);
    }

    /// Passes the event down to the currently selected pane.
    fn pass_event_down(&mut self, app: &mut App, event: Event) -> Result<(), Box<dyn Error>> {
        use BrowsePane::*;
        match self.selected_pane {
            Playlists => self.playlists.handle_event(app, event),
            Songs => self.songs.handle_event(app, event),
            Modal(_) => {
                let msg = self.modal.handle_event(event)?;
                self.handle_modal_message(app, msg)
            }
        }
    }

    /// When a modal handles an event, it returns a message, which can be Nothing, Quit, or
    /// Commit(String). This method handles that message.
    fn handle_modal_message(
        &mut self,
        app: &mut App,
        msg: modal::Message,
    ) -> Result<(), Box<dyn Error>> {
        if let BrowsePane::Modal(modal_type) = &self.selected_pane {
            use modal::Message::*;
            use ModalType::*;
            match (modal_type, msg) {
                (_, Nothing) => {}

                // AddSong
                (AddSong { playlist: _ }, Quit) => {
                    self.selected_pane = BrowsePane::Songs;
                }
                (AddSong { playlist }, Commit(song)) => {
                    playlist_management::add_song(app, playlist, song);
                    self.selected_pane = BrowsePane::Songs;
                }

                // AddPlaylist
                (AddPlaylist, Quit) => {
                    self.selected_pane = BrowsePane::Playlists;
                }
                (AddPlaylist, Commit(playlist)) => {
                    playlist_management::create_playlist(app, &playlist)?;
                    self.playlists.reload_from_dir()?;
                    self.selected_pane = BrowsePane::Playlists;
                }

                // Play
                (Play, Quit) => {
                    self.selected_pane = BrowsePane::Songs;
                }
                (Play, Commit(path)) => {
                    app.mpv
                        .playlist_load_files(&[(&path, libmpv::FileState::Replace, None)])?;
                    self.selected_pane = BrowsePane::Songs;
                }
            }
        } else {
            panic!("Please don't call BrowseScreen::handle_modal_message without a selected modal");
        }
        Ok(())
    }

    /// Handles an Event::Command(cmd)
    fn handle_command(
        &mut self,
        app: &mut App,
        cmd: command::Command,
    ) -> Result<(), Box<dyn Error>> {
        use command::Command::*;
        match cmd {
            Quit => {
                app.change_state(None);
            }
            SeekForward => {
                app.mpv.seek_forward(10.).ok();
                self.now_playing.update(&app.mpv);
            }
            SeekBackward => {
                app.mpv.seek_backward(10.).ok();
                self.now_playing.update(&app.mpv);
            }
            NextSong => {
                app.mpv
                    .playlist_next_weak()
                    .unwrap_or_else(|_| app.notify_err("No next song".into()));
                self.now_playing.update(&app.mpv);
            }
            PrevSong => {
                app.mpv
                    .playlist_previous_weak()
                    .unwrap_or_else(|_| app.notify_err("No previous song".into()));
                self.now_playing.update(&app.mpv);
            }
            TogglePause => {
                app.mpv.command("cycle", &["pause"])?;
                self.now_playing.update(&app.mpv);
            }
            VolumeUp => {
                app.mpv.add_property("volume", 5)?;
                self.now_playing.update(&app.mpv);
            }
            VolumeDown => {
                app.mpv.add_property("volume", -5)?;
                self.now_playing.update(&app.mpv);
            }
            PlayFromModal => {
                self.selected_pane = BrowsePane::Modal(ModalType::Play);
                self.modal = Modal::new(" Play ".into());
            }
            // TODO: this should probably be in each pane's handle_event, somehow
            Add => match self.selected_pane {
                BrowsePane::Playlists => {
                    self.selected_pane = BrowsePane::Modal(ModalType::AddPlaylist);
                    self.modal = Modal::new(" Add playlist ".into());
                }
                BrowsePane::Songs => {
                    if let Some(playlist) = self.playlists.selected_item() {
                        self.selected_pane = BrowsePane::Modal(ModalType::AddSong {
                            playlist: playlist.to_owned(),
                        });
                        self.modal = Modal::new(" Add song ".into());
                    }
                }
                BrowsePane::Modal(_) => {}
            },
            _ => self.pass_event_down(app, Event::Command(cmd))?,
        }
        Ok(())
    }

    /// Handles an Event::Terminal(event)
    fn handle_terminal_event(
        &mut self,
        app: &mut App,
        event: crossterm::event::Event,
    ) -> Result<(), Box<dyn Error>> {
        use BrowsePane::{Playlists, Songs};
        use Event::*;
        use KeyCode::*;

        match event {
            crossterm::event::Event::Key(event) => match event.code {
                Right | Left => {
                    match self.selected_pane {
                        Playlists => {
                            self.selected_pane = Songs;
                        }
                        Songs => {
                            self.selected_pane = Playlists;
                        }
                        BrowsePane::Modal(_) => {}
                    };
                }
                // 'c'hange
                KeyCode::Char('c') if self.mode() == Mode::Normal => {
                    self.playlists.open_editor_for_selected()?;
                }
                _ => self.pass_event_down(app, Terminal(crossterm::event::Event::Key(event)))?,
            },
            _ => {
                self.pass_event_down(app, Terminal(event))?;
            }
        }
        Ok(())
    }
}

impl Screen for BrowseScreen {
    fn render(&mut self, frame: &mut Frame<'_, MyBackend>) {
        let size = frame.size();

        let vchunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(20), Constraint::Length(2)].as_ref())
            .split(size);

        let hchunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(15), Constraint::Percentage(85)].as_ref())
            .split(vchunks[0]);

        self.playlists.render(
            self.selected_pane == BrowsePane::Playlists,
            frame,
            hchunks[0],
        );
        self.songs
            .render(self.selected_pane == BrowsePane::Songs, frame, hchunks[1]);

        self.now_playing.render(frame, vchunks[1]);

        if let BrowsePane::Modal(_) = self.selected_pane {
            self.modal.render(frame);
        }
    }

    fn handle_event(&mut self, app: &mut App, event: Event) -> Result<(), Box<dyn Error>> {
        use Event::*;
        match event {
            Command(cmd) => {
                self.handle_command(app, cmd)?;
            }
            SongAdded { playlist, song } => {
                self.reload_songs();
                app.notify_ok(format!("\"{}\" was added to {}", song, playlist));
            }
            SecondTick => {
                self.now_playing.update(&app.mpv);
            }
            ChangedPlaylist => {
                self.reload_songs();
            }
            Terminal(event) => {
                self.handle_terminal_event(app, event)?;
            }
        }
        Ok(())
    }

    fn mode(&self) -> Mode {
        use BrowsePane::*;
        match self.selected_pane {
            Playlists => self.playlists.mode(),
            Songs => self.songs.mode(),
            Modal(_) => self.modal.mode(),
        }
    }
}
