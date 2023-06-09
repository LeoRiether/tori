use crate::{command, error::Result, events, rect_ops::RectOps};

mod now_playing;
use now_playing::NowPlaying;
use tui::layout::Rect;

use super::{
    browse_screen::BrowseScreen,
    component::{Component, MouseHandler},
    playlist_screen::PlaylistScreen,
    App, Mode,
};

#[derive(Debug, Default)]
pub enum Selected {
    #[default]
    Browse,
    Playlist,
}

#[derive(Debug)]
pub struct AppScreen<'a> {
    browse: BrowseScreen<'a>,
    playlist: PlaylistScreen,
    now_playing: NowPlaying,
    selected: Selected,
}

impl<'a> AppScreen<'a> {
    pub fn new() -> Result<Self> {
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

    pub fn pass_event_down(&mut self, app: &mut App, event: events::Event) -> Result<()> {
        match self.selected {
            Selected::Browse => self.browse.handle_event(app, event),
            Selected::Playlist => self.playlist.handle_event(app, event),
        }
    }

    fn handle_command(&mut self, app: &mut App, cmd: command::Command) -> Result<()> {
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
                    .unwrap_or_else(|_| app.notify_err("No next song"));
                self.now_playing.update(&app.mpv);
            }
            PrevSong => {
                app.mpv
                    .playlist_previous_weak()
                    .unwrap_or_else(|_| app.notify_err("No previous song"));
                self.now_playing.update(&app.mpv);
            }
            TogglePause => {
                app.mpv.command("cycle", &["pause"])?;
                self.now_playing.update(&app.mpv);
            }
            ToggleLoop => {
                let status = app.mpv.get_property::<String>("loop-file");
                let next_status = match status.as_deref() {
                    Ok("no") => "inf",
                    _ => "no",
                };
                app.mpv.set_property("loop-file", next_status)?;
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
            _ => self.pass_event_down(app, events::Event::Command(cmd))?,
        }
        Ok(())
    }

    /// Returns (app chunk, now_playing chunk)
    fn subcomponent_chunks(frame: Rect) -> (Rect, Rect) {
        frame.split_bottom(2)
    }
}

impl<'a> Component for AppScreen<'a> {
    type RenderState = ();

    fn mode(&self) -> Mode {
        match self.selected {
            Selected::Browse => self.browse.mode(),
            Selected::Playlist => self.playlist.mode(),
        }
    }

    fn render(&mut self, frame: &mut tui::Frame<'_, super::MyBackend>, chunk: Rect, (): ()) {
        let vchunks = Self::subcomponent_chunks(chunk);

        match self.selected {
            Selected::Browse => self.browse.render(frame, vchunks.0, ()),
            Selected::Playlist => self.playlist.render(frame, vchunks.0, ()),
        }

        self.now_playing.render(frame, vchunks.1, ());
    }

    fn handle_event(&mut self, app: &mut App, event: events::Event) -> Result<()> {
        use crossterm::event::KeyCode;
        use events::Event::*;
        match &event {
            Command(cmd) => self.handle_command(app, *cmd)?,
            Terminal(crossterm::event::Event::Key(key_event)) => match key_event.code {
                KeyCode::Char('1') if self.mode() == Mode::Normal => {
                    self.select(Selected::Browse);
                }
                KeyCode::Char('2') if self.mode() == Mode::Normal => {
                    self.playlist.update(&app.mpv)?;
                    self.select(Selected::Playlist);
                }
                _ => self.pass_event_down(app, event)?,
            },
            SecondTick => {
                self.now_playing.update(&app.mpv);
                self.pass_event_down(app, event)?;
            }
            _ => self.pass_event_down(app, event)?,
        }
        Ok(())
    }
}

impl<'a> MouseHandler for AppScreen<'a> {
    fn handle_mouse(
        &mut self,
        app: &mut App,
        chunk: Rect,
        event: crossterm::event::MouseEvent,
    ) -> Result<()> {
        let vchunks = Self::subcomponent_chunks(chunk);
        if vchunks.0.contains(event.column, event.row) {
            return match self.selected {
                Selected::Browse => self.browse.handle_mouse(app, vchunks.0, event),
                Selected::Playlist => self.playlist.handle_mouse(app, vchunks.0, event),
            };
        }
        if vchunks.1.contains(event.column, event.row) {
            return self.now_playing.handle_mouse(app, vchunks.1, event);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_big_frame_size() {
        let frame = Rect { x: 0, y: 0, width: 128, height: 64 };
        let app = Rect { x: 0, y: 0, width: 128, height: 62 };
        let now_playing = Rect { x: 0, y: 62, width: 128, height: 2 };
        assert_eq!(AppScreen::subcomponent_chunks(frame), (app, now_playing));
    }

    #[test]
    fn test_small_frame_size() {
        let frame = Rect { x: 0, y: 0, width: 16, height: 10 };
        let app = Rect { x: 0, y: 0, width: 16, height: 8 };
        let now_playing = Rect { x: 0, y: 8, width: 16, height: 2 };
        assert_eq!(AppScreen::subcomponent_chunks(frame), (app, now_playing));
    }

    #[test]
    fn test_unusably_small_frame_size() {
        let frame = Rect { x: 0, y: 0, width: 16, height: 1 };
        let app = Rect { x: 0, y: 0, width: 16, height: 0 };
        let now_playing = Rect { x: 0, y: 0, width: 16, height: 1 };
        assert_eq!(AppScreen::subcomponent_chunks(frame), (app, now_playing));
    }
}

