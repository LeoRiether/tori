pub mod app;
pub mod command;
pub mod config;
pub mod dbglog;
pub mod events;
pub mod m3u;
pub mod shortcuts;

use app::{browse_screen::BrowseScreen, App};
use argh::FromArgs;
use config::{Config, OptionalConfig};
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
    let args: Args = argh::from_env();
    Config::set_global({
        let opt_conf = OptionalConfig::from_path(
            args.config
                .map(PathBuf::from)
                .unwrap_or(dirs::config_dir().unwrap_or_default().join("tori.yaml")),
        )?;

        Config::default().merge(opt_conf)
    });

    let mut app = App::new(BrowseScreen::new()?)?;
    app.run()
}
