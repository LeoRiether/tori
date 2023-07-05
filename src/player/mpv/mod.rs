use crate::config::Config;
use crate::error::Result;

mod select;
use select::Mpv;

#[repr(transparent)]
pub struct MpvPlayer {
    pub(crate) mpv: Mpv,
}

impl super::Player for MpvPlayer {
    fn new() -> Result<Self> {
        let mpv = Mpv::with_initializer(|mpv| {
            mpv.set_bool("video", false)?;
            mpv.set_i64("volume", 100)?;
            if let Some(ao) = &Config::global().mpv_ao {
                mpv.set_str("ao", ao.as_str())?;
            }
            Ok(())
        })?;

        Ok(Self { mpv })
    }

    fn play(&mut self, path: &str) -> Result<()> {
        self.mpv.play(path)?;
        Ok(())
    }

    fn queue(&mut self, path: &str) -> Result<()> {
        self.mpv.queue(path)?;
        Ok(())
    }

    fn seek(&mut self, seconds: f64) -> Result<()> {
        if seconds >= 0.0 {
            self.mpv.seek_forward(seconds)?
        } else {
            self.mpv.seek_backward(-seconds)?
        }
        Ok(())
    }

    fn seek_absolute(&mut self, percent: usize) -> Result<()> {
        self.mpv.seek_percent_absolute(percent)?;
        Ok(())
    }

    fn playlist_next(&mut self) -> Result<()> {
        self.mpv.playlist_next_weak()?;
        Ok(())
    }

    fn playlist_previous(&mut self) -> Result<()> {
        self.mpv.playlist_previous_weak()?;
        Ok(())
    }

    fn toggle_pause(&mut self) -> Result<()> {
        self.mpv.command("cycle", &["pause"])?;
        Ok(())
    }

    fn toggle_loop_file(&mut self) -> Result<()> {
        let status = self.mpv.get_str("loop-file");
        let next_status = match status.as_deref() {
            Ok("no") => "inf",
            _ => "no",
        };
        self.mpv.set_str("loop-file", next_status)?;
        Ok(())
    }

    fn looping_file(&self) -> Result<bool> {
        let status = self.mpv.get_str("loop-file")?;
        Ok(status == "inf")
    }

    fn volume(&self) -> Result<i64> {
        Ok(self.mpv.get_i64("volume")?)
    }

    fn add_volume(&mut self, x: isize) -> Result<()> {
        self.mpv.add_isize("volume", x)?;
        Ok(())
    }

    fn set_volume(&mut self, x: i64) -> Result<()> {
        self.mpv.set_i64("volume", x)?;
        Ok(())
    }

    fn toggle_mute(&mut self) -> Result<()> {
        self.mpv.command("cycle", &["mute"])?;
        Ok(())
    }

    fn muted(&self) -> Result<bool> {
        Ok(self.mpv.get_bool("mute")?)
    }

    fn media_title(&self) -> Result<String> {
        Ok(self.mpv.get_str("media-title")?)
    }

    fn percent_pos(&self) -> Result<i64> {
        Ok(self.mpv.get_i64("percent-pos")?)
    }

    fn time_pos(&self) -> Result<i64> {
        Ok(self.mpv.get_i64("time-pos")?)
    }

    fn time_remaining(&self) -> Result<i64> {
        Ok(self.mpv.get_i64("time-remaining")?)
    }

    fn paused(&self) -> Result<bool> {
        Ok(self.mpv.get_bool("pause")?)
    }

    fn shuffle(&mut self) -> Result<()> {
        Ok(self.mpv.command("playlist-shuffle", &[])?)
    }

    fn playlist_count(&self) -> Result<usize> {
        Ok(self.mpv.get_i64("playlist/count")? as usize)
    }

    fn playlist_track_title(&self, i: usize) -> Result<String> {
        Ok(self
            .mpv
            .get_str(&format!("playlist/{}/title", i))
            .or_else(|_| self.mpv.get_str(&format!("playlist/{}/filename", i)))?)
    }

    fn playlist_position(&self) -> Result<usize> {
        Ok(self.mpv.get_i64("playlist-playing-pos")? as usize)
    }
}
