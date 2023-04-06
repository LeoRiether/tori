use std::error::Error;

use crate::{command, events};

mod now_playing;
use now_playing::NowPlaying;
use tui::layout::{Layout, Direction, Constraint, Rect};

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
    now_playing: NowPlaying,
    selected: Selected,
}

impl AppScreen {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            browse: BrowseScreen::new()?,
            playlist: PlaylistScreen::default(),
            now_playing: NowPlaying::default(),
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

    fn handle_command(
        &mut self,
        app: &mut App,
        cmd: command::Command,
    ) -> Result<(), Box<dyn Error>> {
        use command::Command::*;
        match cmd {
            Quit => {
                app.quit();
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
            Mute => {
                app.mpv.cycle_property("mute", true)?;
                self.now_playing.update(&app.mpv);
            }
            _ => {}
        }
        Ok(())
    }
}

impl Screen for AppScreen {
    fn mode(&self) -> Mode {
        match self.selected {
            Selected::Browse => self.browse.mode(),
            Selected::Playlist => self.playlist.mode(),
        }
    }

    fn render(&mut self, frame: &mut tui::Frame<'_, super::MyBackend>, chunk: Rect) {
        let vchunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(20), Constraint::Length(2)].as_ref())
            .split(chunk);

        match self.selected {
            Selected::Browse => self.browse.render(frame, vchunks[0]),
            Selected::Playlist => self.playlist.render(frame, vchunks[0]),
        }

        self.now_playing.render(frame, vchunks[1]);
    }

    fn handle_event(&mut self, app: &mut App, event: events::Event) -> Result<(), Box<dyn Error>> {
        use crossterm::event::KeyCode;
        use events::Event::*;
        match &event {
            Command(cmd) => self.handle_command(app, *cmd)?,
            Terminal(crossterm::event::Event::Key(key_event)) => {
                match key_event.code {
                    KeyCode::Char('1') => {
                        self.select(Selected::Browse);
                    }
                    KeyCode::Char('2') => {
                        self.playlist.update(&app.mpv)?;
                        self.select(Selected::Playlist);
                    }
                    _ => self.pass_event_down(app, event)?,
                }
            }
            SecondTick => {
                self.now_playing.update(&app.mpv);
                self.pass_event_down(app, event)?;
            }
            _ => self.pass_event_down(app, event)?,
        }
        Ok(())
    }
}
