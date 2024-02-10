use crate::{
    player::Player,
    ui::{self, eventful_widget::Listener, EventfulWidget}, events::Action,
};
use tui::{
    prelude::*,
    widgets::{Paragraph, Widget},
};

/// NowPlaying represents the bar at the bottom of the screen that shows the current media title,
/// volume, playback position, and other information.
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

impl NowPlaying {
    pub fn update(&mut self, player: &impl Player) {
        self.media_title = player.media_title().unwrap_or_default();
        self.percentage = player.percent_pos().unwrap_or_default();
        self.time_pos = player.time_pos().unwrap_or_default();
        self.time_rem = player.time_remaining().unwrap_or_default();
        self.paused = player.paused().unwrap_or_default();
        self.loop_file = player.looping_file().unwrap_or_default();
        self.volume = match player.muted() {
            Ok(false) => player.volume().unwrap_or_default(),
            _ => 0,
        };
    }
}

impl EventfulWidget<Action> for NowPlaying {
    fn render(&mut self, area: Rect, buf: &mut Buffer, l: &mut Vec<Listener<Action>>) {
        let (playback_left_str, playback_right_str) = playback_strs(self);
        let chunks = subcomponent_chunks(self, area);

        ///////////////////////////////
        //        Media title        //
        ///////////////////////////////
        let media_title = {
            let mut parts = vec![];

            if self.paused {
                parts.push(Span::styled(
                    "[paused] ",
                    Style::default().fg(Color::DarkGray),
                ));
            }

            if self.loop_file {
                parts.push(Span::styled(
                    "[looping] ",
                    Style::default().fg(Color::DarkGray),
                ));
            }

            parts.push(Span::styled(
                &self.media_title,
                Style::default().fg(Color::Yellow),
            ));

            Paragraph::new(Line::from(parts)).alignment(Alignment::Center)
        };

        //////////////////////////
        //        Volume        //
        //////////////////////////
        let volume_title = Paragraph::new(Line::from(vec![
            Span::raw("volume "),
            Span::styled(
                format!("{}%", self.volume),
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .alignment(Alignment::Left);

        let volume_paragraph = {
            // NOTE: the maximum volume is actually 130
            // NOTE: (x + 129) / 130 computes the ceiling of x/130
            let left_width = ((self.volume as usize * chunks.volume.width as usize + 129) / 130)
                .saturating_sub(1);
            let left = "─".repeat(left_width);
            let indicator = "■";
            let right = "─"
                .repeat((chunks.volume.width as usize * 100 / 130).saturating_sub(left_width + 1));
            Paragraph::new(Line::from(vec![
                Span::styled(left, Style::default().fg(Color::White)),
                Span::styled(indicator, Style::default().fg(Color::White)),
                Span::styled(right, Style::default().fg(Color::DarkGray)),
            ]))
        };

        ///////////////////////////////////////
        //        Playback percentage        //
        ///////////////////////////////////////
        let playback_bar_str: String = {
            let mut s: Vec<_> = "─"
                .repeat(chunks.playback_bar.width as usize)
                .chars()
                .collect();
            let i = (self.percentage as usize * s.len() / 100)
                .min(s.len() - 1)
                .max(0);
            s[i] = '■';
            s.into_iter().collect()
        };

        let white = Style::default().fg(Color::White);
        let playback_left = Paragraph::new(playback_left_str).style(white);
        let playback_bar = Paragraph::new(playback_bar_str).style(white);
        let playback_right = Paragraph::new(playback_right_str).style(white);

        /////////////////////////////////////
        //        Render everything        //
        /////////////////////////////////////
        volume_title.render(chunks.top_line, buf);
        media_title.render(chunks.top_line, buf);
        volume_paragraph.render(chunks.volume, buf);
        playback_left.render(chunks.playback_left, buf);
        playback_right.render(chunks.playback_right, buf);
        playback_bar.render(chunks.playback_bar, buf);

        //////////////////////////////////////
        //        Register listeners        //
        //////////////////////////////////////
        use ui::{on, UIEvent::*};
        l.extend([
            on(Click(chunks.volume), |_| todo!()),
            on(Click(chunks.playback_bar), |_| todo!()),
        ])
    }
}

struct SubcomponentChunks {
    top_line: Rect,
    volume: Rect,
    playback_left: Rect,
    playback_bar: Rect,
    playback_right: Rect,
}

fn playback_strs(np: &NowPlaying) -> (String, String) {
    let playback_left_str = format!("⏴︎ {:02}:{:02} ", np.time_pos / 60, np.time_pos % 60);
    let playback_right_str = format!("-{:02}:{:02} ⏵︎", np.time_rem / 60, np.time_rem % 60);
    (playback_left_str, playback_right_str)
}

fn subcomponent_chunks(np: &NowPlaying, chunk: Rect) -> SubcomponentChunks {
    let (playback_left_str, playback_right_str) = playback_strs(np);
    let strlen = |s: &str| s.chars().count();

    let lines = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(chunk);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Length(1),
            Constraint::Length(strlen(&playback_left_str) as u16),
            Constraint::Min(5),
            Constraint::Length(strlen(&playback_right_str) as u16),
        ])
        .split(lines[1]);

    SubcomponentChunks {
        top_line: lines[0],
        volume: chunks[0],
        playback_left: chunks[2],
        playback_bar: chunks[3],
        playback_right: chunks[4],
    }
}
