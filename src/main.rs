pub use tori::*;

use app::App;
use argh::FromArgs;
use config::{Config, OptionalConfig};
pub use error::{Error, Result};
use std::path::{Path, PathBuf};

#[derive(FromArgs)]
/// The frictionless music player for the terminal
struct Args {
    #[argh(option, short = 'c')]
    /// the path to an alternative config file. If not present, the config is loaded from
    /// $CONFIG_DIR/tori.yaml, where $CONFIG_DIR is $HOME/.config on Linux,
    /// $HOME/Library/Application Support on macOS, and %appdata% on Windows.
    config: Option<String>,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    Config::set_global({
        let opt_conf = OptionalConfig::from_path(
            args.config
                .map(PathBuf::from)
                .unwrap_or(dirs::config_dir().unwrap_or_default().join("tori.yaml")),
        )?;

        Config::default().merge(opt_conf)
    });

    make_sure_playlist_dir_exists();

    let mut app = App::new()?;
    app.run()
}

fn make_sure_playlist_dir_exists() {
    let dir_str = &Config::global().playlists_dir;
    let dir = Path::new(dir_str);

    if !dir.exists() {
        print!(
            r"It seems your playlist directory ({dir_str}) does not exist!
Would you like to create it? (Y/n) "
        );
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() != "n" {
            std::fs::create_dir_all(dir).unwrap();
        } else {
            println!(
                r"
tori cannot run without a playlists directory!
You can either create the directory manually, or configure another path
for the playlists by editing the config file.
More information can be found in the docs: https://leoriether.github.io/tori/#configuration/"
            );
            std::process::exit(1);
        }
    }

    if dir.is_file() {
        println!(
            r"The path to your playlists directory ({dir_str}) is a file, not a directory!
To avoid data loss, tori will not delete it, but it will also not run until you fix this :)
You can either delete the file and let tori create the directory, or configure another path
for the playlists by editing the config file.
More information can be found in the docs: https://leoriether.github.io/tori/#configuration/"
        );
        std::process::exit(1);
    }
}
