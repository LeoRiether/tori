use self::centered_list::{CenteredList, CenteredListItem, CenteredListState};
use super::{
    component::{Component, MouseHandler},
    App, Mode,
};
use crate::{command, events};
use std::{error::Error, thread, time::Duration};
use tui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};

mod centered_list;

#[derive(Debug, Default)]
pub struct PlaylistScreen {
    songs: Vec<String>,
    playing: CenteredListState,
}

impl PlaylistScreen {
    /// See <https://mpv.io/manual/master/#command-interface-playlist>
    pub fn update(&mut self, mpv: &libmpv::Mpv) -> Result<&mut Self, Box<dyn Error>> {
        let n = mpv.get_property("playlist/count")?;

        self.songs = (0..n)
            .map(|i| {
                mpv.get_property(&format!("playlist/{}/title", i))
                    .or_else(|_| mpv.get_property(&format!("playlist/{}/filename", i)))
            })
            .collect::<Result<_, libmpv::Error>>()?;

        self.playing
            .select(match mpv.get_property::<i64>("playlist-playing-pos").ok() {
                Some(i) if i >= 0 => Some(i as usize),
                _ => None,
            });

        Ok(self)
    }

    /// Waits a couple of milliseconds, then calls [update](PlaylistScreen::update). It's used
    /// primarily by [select_next](PlaylistScreen::select_next) and
    /// [select_prev](PlaylistScreen::select_prev) because mpv takes a while to update the playlist
    /// properties after changing the selection.
    pub fn update_after_delay(&self, app: &App) {
        let sender = app.channel.sender.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(16));
            sender.send(events::Event::SecondTick).unwrap();
        });
    }

    fn handle_command(
        &mut self,
        app: &mut App,
        cmd: command::Command,
    ) -> Result<(), Box<dyn Error>> {
        use command::Command::*;
        match cmd {
            SelectNext | NextSong => self.select_next(app),
            SelectPrev | PrevSong => self.select_prev(app),
            _ => {}
        }
        Ok(())
    }

    fn handle_terminal_event(
        &mut self,
        app: &mut App,
        event: crossterm::event::Event,
    ) -> Result<(), Box<dyn Error>> {
        use crossterm::event::{Event, KeyCode};
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Up => self.select_prev(app),
                KeyCode::Down => self.select_next(app),
                _ => {}
            }
        }
        Ok(())
    }

    fn select_next(&self, app: &mut App) {
        app.mpv
            .playlist_next_weak()
            .unwrap_or_else(|_| app.notify_err("No next song"));
        self.update_after_delay(app);
    }

    fn select_prev(&self, app: &mut App) {
        app.mpv
            .playlist_previous_weak()
            .unwrap_or_else(|_| app.notify_err("No previous song"));
        self.update_after_delay(app);
    }
}

impl Component for PlaylistScreen {
    type RenderState = ();

    fn mode(&self) -> Mode {
        Mode::Normal
    }

    fn render(&mut self, frame: &mut tui::Frame<'_, super::MyBackend>, chunk: Rect, (): ()) {
        let block = Block::default()
            .title(" Playlist ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightRed));

        let items: Vec<_> = self
            .songs
            .iter()
            .map(|x| CenteredListItem::new(x.as_str()))
            .collect();
        let list = CenteredList::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::Red).fg(Color::White))
            .highlight_symbol("›")
            .highlight_symbol_right("‹");

        frame.render_stateful_widget(list, chunk, &mut self.playing);
    }

    fn handle_event(
        &mut self,
        app: &mut App,
        event: events::Event,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use events::Event::*;
        match event {
            Command(cmd) => self.handle_command(app, cmd)?,
            Terminal(event) => self.handle_terminal_event(app, event)?,
            SecondTick => {
                self.update(&app.mpv)?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl MouseHandler for PlaylistScreen {
    fn handle_mouse(
        &mut self,
        _app: &mut App,
        _chunk: Rect,
        _event: crossterm::event::MouseEvent,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
