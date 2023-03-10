

use libmpv::Mpv;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Paragraph},
    Frame,
};

use crate::app::MyBackend;

#[derive(Debug, Default)]
pub struct NowPlaying {
    pub media_title: String,
    pub percentage: i64,
    pub time_pos: i64,
    pub time_rem: i64,
    pub paused: bool,
}

impl NowPlaying {
    pub fn update(&mut self, mpv: &Mpv) {
        self.media_title = mpv.get_property("media-title").unwrap_or_default();
        self.percentage = mpv.get_property("percent-pos").unwrap_or_default();
        self.time_pos = mpv.get_property("time-pos").unwrap_or_default();
        self.time_rem = mpv.get_property("time-remaining").unwrap_or_default();
        self.paused = mpv.get_property("pause").unwrap_or_default();
    }

    pub fn render(&self, frame: &mut Frame<'_, MyBackend>, chunk: Rect) {
        let lines = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(chunk);

        let playback_left_str = format!("⏴︎ {:02}:{:02} ", self.time_pos / 60, self.time_pos % 60);
        let playback_right_str = format!("-{:02}:{:02} ⏵︎", self.time_rem / 60, self.time_rem % 60);
        let strlen = |s: &str| s.chars().count();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Length(strlen(&playback_left_str) as u16),
                Constraint::Min(5),
                Constraint::Length(strlen(&playback_right_str) as u16),
            ])
            .split(lines[1]);

        let media_title = {
            let mut parts = vec![];

            if self.paused {
                parts.push(Span::styled(
                    "[paused] ",
                    Style::default().fg(Color::DarkGray),
                ));
            }

            parts.push(Span::styled(
                &self.media_title,
                Style::default().fg(Color::LightYellow),
            ));

            Paragraph::new(Spans::from(parts)).alignment(Alignment::Center)
        };

        let playback_bar_str: String = {
            let mut s: Vec<_> = "─".repeat(chunks[2].width as usize).chars().collect();
            let i = self.percentage as usize * s.len() / 100;
            s[i] = '■';
            s.into_iter().collect()
        };

        let playback_left =
            Paragraph::new(playback_left_str).style(Style::default().fg(Color::White));
        let playback_bar =
            Paragraph::new(playback_bar_str).style(Style::default().fg(Color::White));
        let playback_right =
            Paragraph::new(playback_right_str).style(Style::default().fg(Color::White));

        frame.render_widget(media_title, lines[0]);
        frame.render_widget(playback_left, chunks[1]);
        frame.render_widget(playback_bar, chunks[2]);
        frame.render_widget(playback_right, chunks[3]);
    }
}
