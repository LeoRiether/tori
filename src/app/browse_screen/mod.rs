use crate::app::MyBackend;
use crate::app::Screen;
use crate::App;

use crossterm::event::{self, KeyCode, KeyModifiers};
use std::error::Error;
use tui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

mod playlists;
use playlists::PlaylistsPane;

mod songs;
use songs::SongsPane;

mod add;
use add::AddPane;

use super::event_channel;
use super::event_channel::ToriEvent;
use super::Mode;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(i8)]
enum BrowsePane {
    #[default]
    Playlists,
    Songs,
    Add,
}

#[derive(Debug, Default)]
pub struct BrowseScreen<'a> {
    playlists: PlaylistsPane<'a>,
    songs: SongsPane<'a>,
    add: AddPane,
    selected_pane: BrowsePane,
}

impl<'a> BrowseScreen<'a> {
    pub fn new() -> Self {
        let playlists = PlaylistsPane::from_dir("playlists");
        let songs = SongsPane::from_playlist_pane(&playlists);
        Self {
            playlists,
            songs,
            ..Default::default()
        }
    }

    /// Passes the event down to the currently selected pane.
    fn pass_event_down(
        &mut self,
        app: &mut App,
        event: crossterm::event::Event,
    ) -> Result<(), Box<dyn Error>> {
        use BrowsePane::*;
        match self.selected_pane {
            Playlists => self.playlists.handle_event(event),
            Songs => self.songs.handle_event(app, event),
            Add => self.add.handle_event(
                app,
                self.playlists.selected_item().map(|s| s.as_str()),
                event,
            ),
        }
    }

    pub fn mode(&self) -> Mode {
        use BrowsePane::*;
        match self.selected_pane {
            Playlists => self.playlists.mode(),
            Songs => self.songs.mode(),
            Add => self.add.mode(),
        }
    }
}

impl Screen for BrowseScreen<'_> {
    fn render(&mut self, frame: &mut Frame<'_, MyBackend>) {
        let size = frame.size();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(15), Constraint::Percentage(85)].as_ref())
            .split(size);

        self.playlists.render(
            self.selected_pane == BrowsePane::Playlists,
            frame,
            chunks[0],
        );
        self.songs
            .render(self.selected_pane == BrowsePane::Songs, frame, chunks[1]);

        if self.selected_pane == BrowsePane::Add {
            self.add.render(frame);
        }
    }

    fn handle_terminal_event(
        &mut self,
        app: &mut App,
        event: event::Event,
    ) -> Result<(), Box<dyn Error>> {
        let has_mod = |event: event::KeyEvent, mods: KeyModifiers| {
            event.modifiers & mods != KeyModifiers::NONE
        };

        use BrowsePane::*;
        use KeyCode::*;
        match event {
            crossterm::event::Event::Key(event) => match event.code {
                Char('d') | Char('c') if has_mod(event, KeyModifiers::CONTROL) => {
                    app.change_state(None);
                    return Ok(());
                }
                Esc | Enter if self.selected_pane == Add => {
                    self.pass_event_down(app, crossterm::event::Event::Key(event))?;
                    self.selected_pane = Songs;
                }
                Up | Down | Enter | Char('k') | Char('j') | Char('e') | Char('n') => {
                    self.pass_event_down(app, crossterm::event::Event::Key(event))?;
                    if let Playlists = self.selected_pane {
                        self.songs = SongsPane::from_playlist_pane(&self.playlists);
                    }
                }
                Right | Left => {
                    self.selected_pane = match self.selected_pane {
                        Playlists => Songs,
                        Songs => Playlists,
                        Add => Add,
                    };
                }
                // 'a'dd
                Char('a') if self.mode() == Mode::Normal => {
                    self.selected_pane = Add;
                    self.add = AddPane::new();
                }
                // 'c'hange
                KeyCode::Char('c') if self.mode() == Mode::Normal => {
                    self.playlists.open_editor_for_selected()
                }
                _ => self.pass_event_down(app, crossterm::event::Event::Key(event))?,
            },
            _ => self.pass_event_down(app, event)?,
        }
        Ok(())
    }

    fn handle_tori_event(
        &mut self,
        _app: &mut App,
        event: event_channel::ToriEvent,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            ToriEvent::SongAdded {
                playlist_name: _,
                song: _,
            } => {
                // Reload songs pane
                self.songs = SongsPane::from_playlist_pane(&self.playlists);
            }
        }
        Ok(())
    }
}
