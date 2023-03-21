pub mod app;
pub mod m3u;
pub mod command;
pub mod events;

use app::{browse_screen::BrowseScreen, App};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new(BrowseScreen::new()?)?;
    app.run()
}
