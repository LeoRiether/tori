use crate::error::Result;

#[cfg(feature = "mpv")]
mod mpv;
#[cfg(feature = "mpv")]
pub type DefaultPlayer = mpv::MpvPlayer;

pub trait Player: Sized {
    fn new() -> Result<Self>;
    fn play(&mut self, path: &str) -> Result<()>;
    fn queue(&mut self, path: &str) -> Result<()>;
    fn seek(&mut self, seconds: f64) -> Result<()>;
    fn seek_absolute(&mut self, percent: usize) -> Result<()>;
    fn playlist_next(&mut self) -> Result<()>;
    fn playlist_previous(&mut self) -> Result<()>;
    fn toggle_pause(&mut self) -> Result<()>;
    fn toggle_loop_file(&mut self) -> Result<()>;
    fn looping_file(&self) -> Result<bool>;
    fn volume(&self) -> Result<i64>;
    fn add_volume(&mut self, x: isize) -> Result<()>;
    fn set_volume(&mut self, x: i64) -> Result<()>;
    fn toggle_mute(&mut self) -> Result<()>;
    fn muted(&self) -> Result<bool>;
    fn media_title(&self) -> Result<String>;
    fn percent_pos(&self) -> Result<i64>;
    fn time_pos(&self) -> Result<i64>;
    fn time_remaining(&self) -> Result<i64>;
    fn paused(&self) -> Result<bool>;
    fn shuffle(&mut self) -> Result<()>;

    // Playlist-related:
    fn playlist_count(&self) -> Result<usize>;
    fn playlist_track_title(&self, i: usize) -> Result<String>;
    fn playlist_position(&self) -> Result<usize>;
}
