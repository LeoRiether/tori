pub mod app;
pub mod m3u;
pub mod command;
pub mod events;
pub mod shortcuts;
pub mod config;

use app::{browse_screen::BrowseScreen, App};
use config::Config;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    Config::set_global(Config::from_default_location()?);
    let mut app = App::new(BrowseScreen::new()?)?;
    app.run()
}
