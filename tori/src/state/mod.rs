pub mod browse_screen;

use std::{borrow::Cow, time::Duration};

use tui::style::Color;

use crate::{
    app::modal::Modal,
    component::{Notification, NowPlaying, Visualizer},
    error::Result,
    player::{DefaultPlayer, Player}, events::{channel::Tx, Action}, ui::Listener,
};

use self::browse_screen::BrowseScreen;

/// Holds most of the state of the application, like a kind of database
pub struct State<'n> {
    pub quit: bool,
    pub listeners: Vec<Listener<Action>>,
    pub player: DefaultPlayer,
    pub screen: Screen,
    pub now_playing: NowPlaying,
    pub notification: Option<Notification<'n>>,
    pub modal: Option<Box<dyn Modal>>,
    pub visualizer: Visualizer,
}

#[derive(Default)]
pub enum Screen {
    #[default]
    None,
    BrowseScreen(BrowseScreen),
}

impl<'n> State<'n> {
    pub fn new(tx: Tx, width: usize) -> Result<Self> {
        Ok(Self {
            quit: false,
            listeners: Vec::new(),
            player: DefaultPlayer::new()?,
            screen: Screen::BrowseScreen(BrowseScreen::new()?),
            now_playing: NowPlaying::default(),
            notification: None,
            modal: None,
            visualizer: Visualizer::new(tx, width),
        })
    }

    pub fn quit(&mut self) {
        self.quit = true;
    }

    ////////////////////////////////
    //        Notification        //
    ////////////////////////////////
    pub fn notify_err(&mut self, err: impl Into<Cow<'n, str>>) {
        self.notification =
            Some(Notification::new(err, Duration::from_secs(5)).colored(Color::LightRed));
    }

    pub fn notify_info(&mut self, info: impl Into<Cow<'n, str>>) {
        self.notification =
            Some(Notification::new(info, Duration::from_secs(4)).colored(Color::LightCyan));
    }

    pub fn notify_ok(&mut self, text: impl Into<Cow<'n, str>>) {
        self.notification =
            Some(Notification::new(text, Duration::from_secs(4)).colored(Color::LightGreen));
    }
}
