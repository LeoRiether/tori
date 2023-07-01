pub mod controller;
pub mod source;
mod output;
mod resampler;

use crate::error::Result;
use controller::Controller;

macro_rules! my_todo {
    () => {
        Ok(Default::default())
    };
}

pub struct SymphoniaPlayer {
    controller: Controller,
}

impl super::Player for SymphoniaPlayer {
    fn new() -> Result<Self> {
        pretty_env_logger::init();
        let controller = Controller::default();
        Ok(Self { controller })
    }

    fn play(&mut self, path: &str) -> Result<()> {
        self.controller.play(path)
    }

    fn queue(&mut self, path: &str) -> Result<()> {
        my_todo!()
    }

    fn seek(&mut self, seconds: f64) -> Result<()> {
        my_todo!()
    }

    fn seek_absolute(&mut self, percent: usize) -> Result<()> {
        my_todo!()
    }

    fn playlist_next(&mut self) -> Result<()> {
        my_todo!()
    }

    fn playlist_previous(&mut self) -> Result<()> {
        my_todo!()
    }

    fn toggle_pause(&mut self) -> Result<()> {
        my_todo!()
    }

    fn toggle_loop_file(&mut self) -> Result<()> {
        my_todo!()
    }

    fn looping_file(&self) -> Result<bool> {
        my_todo!()
    }

    fn volume(&self) -> Result<i64> {
        my_todo!()
    }

    fn add_volume(&mut self, x: isize) -> Result<()> {
        my_todo!()
    }

    fn set_volume(&mut self, x: i64) -> Result<()> {
        my_todo!()
    }

    fn toggle_mute(&mut self) -> Result<()> {
        my_todo!()
    }

    fn muted(&self) -> Result<bool> {
        my_todo!()
    }

    fn media_title(&self) -> Result<String> {
        my_todo!()
    }

    fn percent_pos(&self) -> Result<i64> {
        my_todo!()
    }

    fn time_pos(&self) -> Result<i64> {
        my_todo!()
    }

    fn time_remaining(&self) -> Result<i64> {
        my_todo!()
    }

    fn paused(&self) -> Result<bool> {
        my_todo!()
    }

    fn shuffle(&mut self) -> Result<()> {
        my_todo!()
    }

    fn playlist_count(&self) -> Result<usize> {
        my_todo!()
    }

    fn playlist_track_title(&self, i: usize) -> Result<String> {
        my_todo!()
    }

    fn playlist_position(&self) -> Result<usize> {
        my_todo!()
    }
}
