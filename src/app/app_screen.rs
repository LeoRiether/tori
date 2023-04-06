use std::error::Error;

use crate::events;

use super::{browse_screen::BrowseScreen, playlist_screen::PlaylistScreen, App, Mode, Screen};

#[derive(Debug, Default)]
pub enum Selected {
    #[default]
    Browse,
    Playlist,
}

#[derive(Debug)]
pub struct AppScreen {
    browse: BrowseScreen,
    playlist: PlaylistScreen,
    selected: Selected,
}

impl AppScreen {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            browse: BrowseScreen::new()?,
            playlist: PlaylistScreen::default(),
            selected: Selected::default(),
        })
    }

    pub fn select(&mut self, selection: Selected) {
        self.selected = selection;
    }

    pub fn pass_event_down(
        &mut self,
        app: &mut App,
        event: events::Event,
    ) -> Result<(), Box<dyn Error>> {
        match self.selected {
            Selected::Browse => self.browse.handle_event(app, event),
            Selected::Playlist => self.playlist.handle_event(app, event),
        }
    }
}

impl Screen for AppScreen {
    fn mode(&self) -> Mode {
        match self.selected {
            Selected::Browse => self.browse.mode(),
            Selected::Playlist => self.playlist.mode(),
        }
    }

    fn render(&mut self, frame: &mut tui::Frame<'_, super::MyBackend>) {
        match self.selected {
            Selected::Browse => self.browse.render(frame),
            Selected::Playlist => self.playlist.render(frame),
        }
    }

    fn handle_event(&mut self, app: &mut App, event: events::Event) -> Result<(), Box<dyn Error>> {
        use crossterm::event::KeyCode;
        match &event {
            events::Event::Terminal(crossterm::event::Event::Key(key_event)) => match key_event.code {
                KeyCode::Char('1') => {
                    self.select(Selected::Browse);
                }
                KeyCode::Char('2') => {
                    self.playlist.update(&app.mpv)?;
                    self.select(Selected::Playlist);
                }
                _ => self.pass_event_down(app, event)?,
            }
            _ => self.pass_event_down(app, event)?,
        }
        Ok(())
    }
}
