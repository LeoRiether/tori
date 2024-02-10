use crate::{error::Result, events, player::Player, rect_ops::RectOps};

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
    selected: Selected,
}

impl<'a> AppScreen<'a> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            browse: BrowseScreen::new()?,
            playlist: PlaylistScreen::default(),
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

    fn handle_command(&mut self, app: &mut App, cmd: events::Command) -> Result<()> {
        use events::Command::*;
        match cmd {
            Quit => {
            }
            SeekForward => {
                app.state.player.seek(10.)?;
                
            }
            SeekBackward => {
                app.state.player.seek(-10.)?;
                
            }
            NextSong => {
                app.state.player
                    .playlist_next()
                    .unwrap_or_else(|_| app.state.notify_err("No next song"));
                
            }
            PrevSong => {
                app.state.player
                    .playlist_previous()
                    .unwrap_or_else(|_| app.state.notify_err("No previous song"));
                
            }
            TogglePause => {
                app.state.player.toggle_pause()?;
                
            }
            ToggleLoop => {
                app.state.player.toggle_loop_file()?;
                
            }
            VolumeUp => {
                app.state.player.add_volume(5)?;
                
            }
            VolumeDown => {
                app.state.player.add_volume(-5)?;
                
            }
            Mute => {
                app.state.player.toggle_mute()?;
                
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

    fn render(&mut self, frame: &mut tui::Frame, chunk: Rect, (): ()) {
        let vchunks = Self::subcomponent_chunks(chunk);

        match self.selected {
            Selected::Browse => self.browse.render(frame, vchunks.0, ()),
            Selected::Playlist => self.playlist.render(frame, vchunks.0, ()),
        }

        
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
                    self.playlist.update(&app.state.player)?;
                    self.select(Selected::Playlist);
                }
                _ => self.pass_event_down(app, event)?,
            },
            Tick => {
                
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
            return Ok(()) // ?
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_big_frame_size() {
        let frame = Rect {
            x: 0,
            y: 0,
            width: 128,
            height: 64,
        };
        let app = Rect {
            x: 0,
            y: 0,
            width: 128,
            height: 62,
        };
        let now_playing = Rect {
            x: 0,
            y: 62,
            width: 128,
            height: 2,
        };
        assert_eq!(AppScreen::subcomponent_chunks(frame), (app, now_playing));
    }

    #[test]
    fn test_small_frame_size() {
        let frame = Rect {
            x: 0,
            y: 0,
            width: 16,
            height: 10,
        };
        let app = Rect {
            x: 0,
            y: 0,
            width: 16,
            height: 8,
        };
        let now_playing = Rect {
            x: 0,
            y: 8,
            width: 16,
            height: 2,
        };
        assert_eq!(AppScreen::subcomponent_chunks(frame), (app, now_playing));
    }

    #[test]
    fn test_unusably_small_frame_size() {
        let frame = Rect {
            x: 0,
            y: 0,
            width: 16,
            height: 1,
        };
        let app = Rect {
            x: 0,
            y: 0,
            width: 16,
            height: 0,
        };
        let now_playing = Rect {
            x: 0,
            y: 0,
            width: 16,
            height: 1,
        };
        assert_eq!(AppScreen::subcomponent_chunks(frame), (app, now_playing));
    }
}
