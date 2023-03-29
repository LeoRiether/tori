pub mod app;
pub mod command;
pub mod config;
pub mod events;
pub mod m3u;
pub mod shortcuts;

use app::{browse_screen::BrowseScreen, App};
use argh::FromArgs;
use config::Config;
use std::{error::Error, path::PathBuf};

#[derive(FromArgs)]
/// Terminal-based music player
struct Args {
    #[argh(option, short = 'c')]
    /// the path to an alternative config file. If not present, the config is loaded from
    /// $CONFIG_DIR/tori.yaml, where $CONFIG_DIR is $HOME/.config on Linux,
    /// $HOME/Library/Application Support on macOS, and %appdata% on Windows.
    config: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    Config::set_global({
        let args: Args = argh::from_env();
        let default_config_path = || dirs::config_dir().unwrap_or_default().join("tori.yaml");
        Config::from_path(
            args.config
                .map(PathBuf::from)
                .unwrap_or_else(default_config_path),
        )?
    });

    let mut app = App::new(BrowseScreen::new()?)?;
    app.run()
}
