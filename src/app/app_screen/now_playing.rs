use libmpv::Mpv;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Line},
    widgets::Paragraph,
    Frame,
};

use crate::{
    app::{
        component::{Component, Mode, MouseHandler},
        App, MyBackend,
    },
    error::Result,
    events,
    rect_ops::RectOps,
};

#[derive(Debug)]
struct SubcomponentChunks {
    top_line: Rect,
    volume: Rect,
    playback_left: Rect,
    playback_bar: Rect,
    playback_right: Rect,
}

#[derive(Debug, Default)]
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
    pub fn update(&mut self, mpv: &Mpv) {
        self.media_title = mpv.get_property("media-title").unwrap_or_default();
        self.percentage = mpv.get_property("percent-pos").unwrap_or_default();
        self.time_pos = mpv.get_property("time-pos").unwrap_or_default();
        self.time_rem = mpv.get_property("time-remaining").unwrap_or_default();
        self.paused = mpv.get_property("pause").unwrap_or_default();

        let loop_file = mpv.get_property::<String>("loop-file");
        self.loop_file = matches!(loop_file.as_deref(), Ok("inf"));

        self.volume = if mpv.get_property("mute").unwrap_or(false) {
            0
        } else {
            mpv.get_property("volume").unwrap_or_default()
        };
    }

    fn playback_strs(&self) -> (String, String) {
        let playback_left_str = format!("⏴︎ {:02}:{:02} ", self.time_pos / 60, self.time_pos % 60);
        let playback_right_str = format!("-{:02}:{:02} ⏵︎", self.time_rem / 60, self.time_rem % 60);
        (playback_left_str, playback_right_str)
    }

    pub fn click(&mut self, app: &mut App, x: u16, y: u16) -> Result<()> {
        let frame = app.frame_size();
        let chunks = self.subcomponent_chunks(frame);

        if chunks.volume.contains(x, y) {
            let dx = (x - chunks.volume.left()) as f64;
            let percentage = dx / chunks.volume.width as f64;
            // remember that the maximum volume is 130 :)
            app.mpv
                .set_property("volume", (130.0 * percentage).round())?;
        }

        if chunks.playback_bar.contains(x, y) {
            let dx = (x - chunks.playback_bar.left()) as f64;
            let percentage = dx / chunks.playback_bar.width as f64;
            let percentage = (percentage * 100.0).round() as usize;
            // this is bugged currently :/
            // app.mpv.seek_percent_absolute(percentage)?;
            app.mpv
                .command("seek", &[&format!("{}", percentage), "absolute-percent"])?;
        }

        self.update(&app.mpv);
        Ok(())
    }

    fn subcomponent_chunks(&self, chunk: Rect) -> SubcomponentChunks {
        let (playback_left_str, playback_right_str) = self.playback_strs();
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
}

impl Component for NowPlaying {
    type RenderState = ();

    fn render(&mut self, frame: &mut Frame<'_, MyBackend>, chunk: Rect, (): ()) {
        let chunks = self.subcomponent_chunks(chunk);
        let (playback_left_str, playback_right_str) = self.playback_strs();

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

        let playback_left =
            Paragraph::new(playback_left_str).style(Style::default().fg(Color::White));
        let playback_bar =
            Paragraph::new(playback_bar_str).style(Style::default().fg(Color::White));
        let playback_right =
            Paragraph::new(playback_right_str).style(Style::default().fg(Color::White));

        /////////////////////////////////////
        //        Render everything        //
        /////////////////////////////////////
        frame.render_widget(volume_title, chunks.top_line);
        frame.render_widget(media_title, chunks.top_line);
        frame.render_widget(volume_paragraph, chunks.volume);
        frame.render_widget(playback_left, chunks.playback_left);
        frame.render_widget(playback_right, chunks.playback_right);
        frame.render_widget(playback_bar, chunks.playback_bar);
    }

    fn mode(&self) -> Mode {
        Mode::Normal
    }

    fn handle_event(&mut self, _app: &mut App, _event: events::Event) -> Result<()> {
        Ok(())
    }
}

impl MouseHandler for NowPlaying {
    fn handle_mouse(
        &mut self,
        app: &mut App,
        _chunk: Rect,
        event: crossterm::event::MouseEvent,
    ) -> Result<()> {
        use crossterm::event::{MouseButton, MouseEventKind};
        if matches!(
            event.kind,
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left)
        ) {
            self.click(app, event.column, event.row)?;
        }
        Ok(())
    }
}
