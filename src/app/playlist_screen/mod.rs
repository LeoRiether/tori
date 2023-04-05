use std::error::Error;

use super::{App, Mode, Screen};

#[derive(Debug, Default)]
pub struct PlaylistScreen {
    songs: Vec<String>,
}

impl PlaylistScreen {
    /// See https://mpv.io/manual/master/#command-interface-playlist
    pub fn from_mpv(mpv: &libmpv::Mpv) -> Result<Self, Box<dyn Error>> {
        let n = mpv.get_property("playlist/count")?;
        let songs = (0..n)
            .map(|i| mpv.get_property(&format!("playlist/{}/title", i)))
            .collect::<Result<_, libmpv::Error>>()?;
        Ok(Self { songs })
    }
}

impl Screen for PlaylistScreen {
    fn mode(&self) -> Mode {
        Mode::Normal
    }

    fn render(&mut self, frame: &mut tui::Frame<'_, super::MyBackend>) {
        todo!()
    }

    fn handle_event(
        &mut self,
        app: &mut App,
        event: crate::events::Event,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}
