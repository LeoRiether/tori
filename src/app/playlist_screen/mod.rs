use std::error::Error;

use tui::{
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState}, layout::Alignment,
};

use crate::{command, events};

use super::{App, Mode, Screen};

#[derive(Debug, Default)]
pub struct PlaylistScreen {
    songs: Vec<String>,
    playing: ListState,
}

impl PlaylistScreen {
    /// See https://mpv.io/manual/master/#command-interface-playlist
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
                KeyCode::Down => self.select_prev(app),
                _ => {}
            }
        }
        Ok(())
    }

    fn select_next(&self, app: &mut App) {
        app.mpv
            .playlist_next_weak()
            .unwrap_or_else(|_| app.notify_err("No next song".into()));
    }

    fn select_prev(&self, app: &mut App) {
        app.mpv
            .playlist_previous_weak()
            .unwrap_or_else(|_| app.notify_err("No next song".into()));
    }
}

impl Screen for PlaylistScreen {
    fn mode(&self) -> Mode {
        Mode::Normal
    }

    fn render(&mut self, frame: &mut tui::Frame<'_, super::MyBackend>) {
        let size = frame.size();

        let block = Block::default()
            .title(" Playlist ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightRed));

        let items: Vec<_> = self
            .songs
            .iter()
            .map(|x| ListItem::new(x.as_str()))
            .collect();
        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::Red).fg(Color::White));

        frame.render_stateful_widget(list, size, &mut self.playing);
    }

    fn handle_event(
        &mut self,
        app: &mut App,
        event: crate::events::Event,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            events::Event::Command(cmd) => self.handle_command(app, cmd)?,
            events::Event::Terminal(event) => self.handle_terminal_event(app, event)?,
            events::Event::SecondTick => {
                self.update(&app.mpv)?;
            }
            _ => {}
        }
        Ok(())
    }
}
