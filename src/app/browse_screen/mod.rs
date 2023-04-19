use crate::{
    app::{component::Component, MyBackend},
    command,
    events::Event,
    m3u::playlist_management,
    util::RectContains,
    App,
};

use crossterm::event::{KeyCode, MouseEvent, MouseEventKind};

use std::borrow::Cow;
use std::error::Error;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Style;
use tui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

mod playlists;
use playlists::PlaylistsPane;

mod songs;
use songs::SongsPane;

use super::modal::{self, ConfirmationModal, HelpModal, InputModal, Modal};
use super::Mode;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ModalType {
    Help,
    Play,
    AddSong { playlist: String },
    AddPlaylist,
    DeletePlaylist { playlist: String },
    RenameSong { playlist: String, index: usize },
    DeleteSong { playlist: String, index: usize },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[repr(i8)]
enum BrowsePane {
    #[default]
    Playlists,
    Songs,
    Modal(ModalType),
}

#[derive(Default)]
pub struct BrowseScreen<'a> {
    playlists: PlaylistsPane,
    songs: SongsPane<'a>,
    modal: Box<dyn Modal>,
    selected_pane: BrowsePane,
}

impl<'a> std::fmt::Debug for BrowseScreen<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowseScreen")
            .field("playlists", &self.playlists)
            .field("songs", &self.songs)
            .field("selected_pane", &self.selected_pane)
            .finish_non_exhaustive()
    }
}

impl<'a> BrowseScreen<'a> {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let playlists = PlaylistsPane::new()?;
        let mut songs = SongsPane::default();
        songs.update_from_playlist_pane(&playlists)?;
        Ok(Self {
            playlists,
            songs,
            ..Default::default()
        })
    }

    pub fn reload_songs(&mut self) -> Result<(), Box<dyn Error>> {
        self.songs.update_from_playlist_pane(&self.playlists)
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

                (Help, _) => {
                    self.selected_pane = BrowsePane::Songs;
                }

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
                    use playlist_management::CreatePlaylistError;
                    match playlist_management::create_playlist(&playlist) {
                        Ok(_) => {
                            self.playlists.reload_from_dir()?;
                        }
                        Err(CreatePlaylistError::PlaylistAlreadyExists) => {
                            app.notify_err(format!("Playlist '{}' already exists!", playlist));
                        }
                        Err(CreatePlaylistError::InvalidChar(c)) => {
                            app.notify_err(format!("Playlist names cannot contain '{}'", c));
                        }
                        Err(CreatePlaylistError::IOError(e)) => return Err(e.into()),
                    }
                    self.selected_pane = BrowsePane::Playlists;
                }

                // DeletePlaylist
                (DeletePlaylist { playlist: _ }, Quit) => {
                    self.selected_pane = BrowsePane::Playlists;
                }
                (DeletePlaylist { playlist }, Commit(_)) => {
                    playlist_management::delete_playlist(playlist)?;
                    self.playlists = PlaylistsPane::new()?;
                    self.reload_songs()?;
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

                // RenameSong
                (
                    RenameSong {
                        playlist: _,
                        index: _,
                    },
                    Quit,
                ) => {
                    self.selected_pane = BrowsePane::Songs;
                }
                (RenameSong { playlist, index }, Commit(new_name)) => {
                    playlist_management::rename_song(playlist, *index, &new_name)?;
                    self.reload_songs()?;
                    self.selected_pane = BrowsePane::Songs;
                }

                // DeleteSong
                (
                    DeleteSong {
                        playlist: _,
                        index: _,
                    },
                    Quit,
                ) => {
                    self.selected_pane = BrowsePane::Songs;
                }
                (DeleteSong { playlist, index }, Commit(_)) => {
                    playlist_management::delete_song(playlist, *index)?;
                    self.reload_songs()?;
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
            PlayFromModal => {
                self.open_modal(" Play ", ModalType::Play);
            }
            OpenHelpModal => {
                self.open_help_modal();
            }
            SelectRight | SelectLeft => self.select_next_panel(),
            // TODO: this should probably be in each pane's handle_event, somehow
            Add => match self.selected_pane {
                BrowsePane::Playlists => {
                    self.open_modal(" Add playlist ", ModalType::AddPlaylist);
                }
                BrowsePane::Songs => {
                    if let Some(playlist) = self.playlists.selected_item() {
                        self.open_modal(
                            " Add song ",
                            ModalType::AddSong {
                                playlist: playlist.to_owned(),
                            },
                        );
                    } else {
                        app.notify_err("Please select a playlist before adding a song");
                    }
                }
                BrowsePane::Modal(_) => {}
            },
            Rename => match self.selected_pane {
                BrowsePane::Playlists => {}
                BrowsePane::Songs => {
                    if let (Some(playlist), Some(index)) =
                        (self.playlists.selected_item(), self.songs.selected_index())
                    {
                        // kind of a hack, sorry
                        // couldn't figure out how to downcast Box<dyn Modal> to Box<InputModal>
                        self.open_modal(
                            "<placeholder>",
                            ModalType::RenameSong {
                                playlist: playlist.to_owned(),
                                index,
                            },
                        );

                        let song_title = self.songs.selected_item().unwrap().title.clone();
                        self.modal = Box::new(
                            InputModal::new(" Rename song (esc cancels) ").set_input(song_title),
                        );
                    }
                }
                _ => {}
            },
            Delete => match self.selected_pane {
                BrowsePane::Playlists => {
                    if let Some(playlist) = self.playlists.selected_item() {
                        let title = format!("Do you really want to delete '{}'?", playlist);
                        let modal_type = ModalType::DeletePlaylist {
                            playlist: playlist.to_owned(),
                        };
                        self.open_confirmation(title.as_str(), modal_type)
                            .apply_style(Style::default().fg(Color::LightRed));
                    }
                }
                BrowsePane::Songs => {
                    if let (Some(playlist), Some(index)) =
                        (self.playlists.selected_item(), self.songs.selected_index())
                    {
                        let title = format!(
                            "Do you really want to delete '{}'?",
                            self.songs.selected_item().unwrap().title
                        );
                        let modal_type = ModalType::DeleteSong {
                            playlist: playlist.to_owned(),
                            index,
                        };
                        self.open_confirmation(title.as_str(), modal_type)
                            .apply_style(Style::default().fg(Color::LightRed));
                    }
                }
                _ => {}
            },
            OpenInEditor => self.playlists.open_editor_for_selected(app)?,
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
        use Event::*;
        use KeyCode::*;

        if let BrowsePane::Modal(_) = self.selected_pane {
            return self.pass_event_down(app, Terminal(event));
        }

        match event {
            crossterm::event::Event::Key(event) => match event.code {
                Right | Left => self.select_next_panel(),
                _ => self.pass_event_down(app, Terminal(crossterm::event::Event::Key(event)))?,
            },
            crossterm::event::Event::Mouse(mouse) => self.handle_mouse(app, mouse)?,
            _ => self.pass_event_down(app, Terminal(event))?,
        }
        Ok(())
    }

    fn handle_mouse(&mut self, app: &mut App, mouse: MouseEvent) -> Result<(), Box<dyn Error>> {
        let event = Event::Terminal(crossterm::event::Event::Mouse(mouse));

        if let BrowsePane::Modal(_) = self.selected_pane {
            return self.pass_event_down(app, event);
        }

        let hchunks = self.subcomponent_chunks(app.frame_size());
        if hchunks[0].contains(mouse.column, mouse.row) {
            if let MouseEventKind::Down(_) = mouse.kind {
                self.selected_pane = BrowsePane::Playlists;
            }
            self.playlists.handle_event(app, event)
        } else {
            if let MouseEventKind::Down(_) = mouse.kind {
                self.selected_pane = BrowsePane::Songs;
            }
            self.songs.handle_event(app, event)
        }
    }

    // TODO: I don't know how to make this 'a instead of 'static :(
    fn open_modal<T>(&mut self, title: T, modal_type: ModalType) -> &mut Box<dyn Modal>
    where
        T: Into<Cow<'static, str>>,
    {
        self.selected_pane = BrowsePane::Modal(modal_type);
        self.modal = Box::new(InputModal::new(title));
        &mut self.modal
    }

    fn open_confirmation(&mut self, title: &str, modal_type: ModalType) -> &mut Box<dyn Modal> {
        self.selected_pane = BrowsePane::Modal(modal_type);
        self.modal = Box::new(ConfirmationModal::new(title));
        &mut self.modal
    }

    fn open_help_modal(&mut self) -> &mut Box<dyn Modal> {
        self.selected_pane = BrowsePane::Modal(ModalType::Help);
        self.modal = Box::new(HelpModal::new());
        &mut self.modal
    }

    fn select_next_panel(&mut self) {
        use BrowsePane::*;
        match self.selected_pane {
            Playlists => {
                self.selected_pane = Songs;
            }
            Songs => {
                self.selected_pane = Playlists;
            }
            Modal(_) => {}
        }
    }

    fn subcomponent_chunks(&self, chunk: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(15), Constraint::Percentage(85)].as_ref())
            .split(chunk)
    }
}

impl<'t> Component for BrowseScreen<'t> {
    type RenderState = ();

    fn render(&mut self, frame: &mut Frame<'_, MyBackend>, chunk: Rect, (): ()) {
        let hchunks = self.subcomponent_chunks(chunk);

        self.playlists.render(
            frame,
            hchunks[0],
            self.selected_pane == BrowsePane::Playlists,
        );
        self.songs
            .render(frame, hchunks[1], self.selected_pane == BrowsePane::Songs);

        if let BrowsePane::Modal(_) = self.selected_pane {
            self.modal.render(frame);
        }
    }

    fn handle_event(&mut self, app: &mut App, event: Event) -> Result<(), Box<dyn Error>> {
        use Event::*;
        match event {
            Command(cmd) => self.handle_command(app, cmd)?,
            SongAdded { playlist, song } => {
                if self.playlists.selected_item() == Some(playlist.as_str()) {
                    self.reload_songs()?;
                }
                app.notify_ok(format!("\"{}\" was added to {}", song, playlist));
            }
            SecondTick => {}
            ChangedPlaylist => {
                self.reload_songs()?;
            }
            Terminal(event) => self.handle_terminal_event(app, event)?,
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
