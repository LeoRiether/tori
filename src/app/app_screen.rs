use std::error::Error;

use super::{browse_screen::BrowseScreen, Screen};

#[derive(Debug, Default)]
pub enum Selected {
    #[default]
    Browse,
}

#[derive(Debug, Default)]
pub struct AppScreen {
    browse: BrowseScreen,
    selected: Selected,
}

impl AppScreen {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            browse: BrowseScreen::new()?,
            ..Default::default()
        })
    }

    pub fn select(&mut self, selection: Selected) {
        self.selected = selection;
    }
}

impl Screen for AppScreen {
    fn mode(&self) -> super::Mode {
        match self.selected {
            Selected::Browse => self.browse.mode(),
        }
    }

    fn render(&mut self, frame: &mut tui::Frame<'_, super::MyBackend>) {
        match self.selected {
            Selected::Browse => self.browse.render(frame),
        }
    }

    fn handle_event(
        &mut self,
        app: &mut super::App,
        event: crate::events::Event,
    ) -> Result<(), Box<dyn Error>> {
        match self.selected {
            Selected::Browse => self.browse.handle_event(app, event),
        }
    }
}
