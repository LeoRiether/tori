pub mod app;
pub mod command;
pub mod events;
pub mod m3u;

use app::{browse_screen::BrowseScreen, App};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new(BrowseScreen::new()?)?;
    app.run()
}
