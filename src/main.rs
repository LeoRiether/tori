pub mod m3u;
pub mod app;

use app::{App, browse_screen::BrowseScreen};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new(BrowseScreen::new()?)?;
    app.run()
}
