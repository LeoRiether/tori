use std::{fs::File, io::BufReader};

use crate::error::Result;

macro_rules! my_todo {
    () => {
        Ok(Default::default())
    };
}

#[derive(Debug, Default, Clone, Copy)]
enum MutedState {
    /// rodio::Sink controls the volume
    #[default]
    Unmuted,
    /// rodio::Sink has volume zero, but we remember the previous volume
    Muted(f32),
}

pub struct RodioPlayer {
    _stream: rodio::OutputStream,
    sink: rodio::Sink,
    muted: MutedState,
}

impl super::Player for RodioPlayer {
    fn new() -> Result<Self> {
        let (stream, handle) = rodio::OutputStream::try_default()?;
        let sink = rodio::Sink::try_new(&handle)?;

        Ok(RodioPlayer {
            _stream: stream,
            sink,
            muted: MutedState::default(),
        })
    }

    fn play(&mut self, path: &str) -> Result<()> {
        self.sink.stop();
        self.queue(path)?;
        Ok(())
    }

    fn queue(&mut self, path: &str) -> Result<()> {
        let file = File::open(path)?;
        let source = rodio::Decoder::new(BufReader::new(file))?;
        self.sink.append(source);
        Ok(())
    }

    fn seek(&mut self, seconds: f64) -> Result<()> {
        my_todo!()
    }

    fn seek_absolute(&mut self, percent: usize) -> Result<()> {
        my_todo!()
    }

    fn playlist_next(&mut self) -> Result<()> {
        self.sink.skip_one();
        Ok(())
    }

    fn playlist_previous(&mut self) -> Result<()> {
        my_todo!()
    }

    fn toggle_pause(&mut self) -> Result<()> {
        if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }
        Ok(())
    }

    fn toggle_loop_file(&mut self) -> Result<()> {
        my_todo!()
    }

    fn looping_file(&self) -> Result<bool> {
        Ok(false)
    }

    fn volume(&self) -> Result<i64> {
        Ok((self.sink.volume() * 100.0).round() as i64)
    }

    fn add_volume(&mut self, x: isize) -> Result<()> {
        let volume = self.sink.volume() + (x as f32 / 100.0);
        self.sink.set_volume(volume);
        Ok(())
    }

    fn set_volume(&mut self, x: i64) -> Result<()> {
        self.sink.set_volume(x as f32 / 100.0);
        Ok(())
    }

    fn toggle_mute(&mut self) -> Result<()> {
        match self.muted {
            MutedState::Unmuted => {
                let volume = self.sink.volume();
                self.sink.set_volume(0.0);
                self.muted = MutedState::Muted(volume);
            }
            MutedState::Muted(volume) => {
                self.sink.set_volume(volume);
                self.muted = MutedState::Unmuted;
            }
        };
        Ok(())
    }

    fn muted(&self) -> Result<bool> {
        Ok(matches!(self.muted, MutedState::Muted(_)))
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
        Ok(self.sink.is_paused())
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
