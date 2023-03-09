use crate::app::MyBackend;
use crate::app::Screen;
use crate::App;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::error::Error;
use tui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

mod playlists;
use playlists::PlaylistsPane;

mod songs;
use songs::SongsPane;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(i8)]
enum BrowsePane {
    #[default]
    Playlists,
    Songs,
    // Add,
}

#[derive(Debug, Default)]
pub struct BrowseScreen<'a> {
    playlists: PlaylistsPane<'a>,
    songs: SongsPane<'a>,
    // add: AddPane,
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
    fn pass_event_down(&mut self, app: &mut App, event: Event) -> Result<(), Box<dyn Error>> {
        use BrowsePane::*;
        match self.selected_pane {
            Playlists => self.playlists.handle_event(event),
            Songs => self.songs.handle_event(app, event),
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
    }

    fn handle_event(&mut self, app: &mut App, event: Event) -> Result<(), Box<dyn Error>> {
        let has_mod = |event: event::KeyEvent, mods: KeyModifiers| {
            event.modifiers & mods != KeyModifiers::NONE
        };

        use BrowsePane::*;
        use KeyCode::*;
        match event {
            Event::Key(event) => {
                match event.code {
                    Char('d') | Char('c') if has_mod(event, KeyModifiers::CONTROL) => {
                        app.change_state(None);
                        return Ok(());
                    }
                    // HACK: for some reason, ctrl+backspace is actually sending a ctrl+h
                    Char('h') if has_mod(event, KeyModifiers::CONTROL) => {}
                    Up | Down | Enter | Char('k') | Char('j') | Char('e') | Char('n') => {
                        self.pass_event_down(app, Event::Key(event))?;
                        if let Playlists = self.selected_pane {
                            self.songs = SongsPane::from_playlist_pane(&self.playlists);
                        }
                    }
                    Right | Left => {
                        self.selected_pane = match self.selected_pane {
                            Playlists => Songs,
                            Songs => Playlists,
                        };
                    }
                    _ => self.pass_event_down(app, Event::Key(event))?,
                }
            }
            _ => self.pass_event_down(app, event)?,
        }
        Ok(())
    }
}
