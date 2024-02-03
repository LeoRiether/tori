pub mod browse_screen;

use std::{borrow::Cow, time::Instant};

use tui::style::Color;

use crate::error::Result;
use crate::{
    app::modal::Modal,
    player::{DefaultPlayer, Player},
    visualizer::Visualizer,
};

use self::browse_screen::BrowseScreen;

/// Holds most of the state of the application, like a kind of database
pub struct State<'n> {
    pub player: DefaultPlayer,
    pub screen: Screen,
    pub now_playing: NowPlaying,
    pub notification: Option<Notification<'n>>,
    pub modal: Option<Box<dyn Modal>>,
    pub visualizer: Option<Visualizer>,
}

pub enum Screen {
    BrowseScreen(BrowseScreen),
}

#[derive(Default)]
pub struct NowPlaying {
    pub media_title: String,
    pub percentage: i64,
    pub time_pos: i64,
    pub time_rem: i64,
    pub paused: bool,
    pub loop_file: bool,
    pub volume: i64,
}

pub struct Notification<'t> {
    pub text: Cow<'t, str>,
    pub show_until: Instant,
    pub color: Color,
    pub height: u16,
}

impl<'n> State<'n> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            player: DefaultPlayer::new()?,
            screen: Screen::BrowseScreen(BrowseScreen::new()?),
            now_playing: NowPlaying::default(),
            notification: None,
            modal: None,
            visualizer: None,
        })
    }
}
