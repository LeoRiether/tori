use super::source;

use crate::error::Result;

#[derive(Debug, Default)]
pub struct Controller {}

impl Controller {
    pub fn play(&mut self, path: &str) -> Result<()> {
        source::start_player_thread(path);
        Ok(())
    }
}
