use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{error::Error, io, path::PathBuf};

use crate::shortcuts::Shortcuts;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub playlists_dir: String,
    pub visualizer_gradient: [(u8, u8, u8); 2],
    pub normal: Shortcuts,
}

static INSTANCE: OnceCell<Config> = OnceCell::new();

impl Config {
    pub fn global() -> &'static Self {
        INSTANCE.get().expect("Config instance not loaded!")
    }

    pub fn set_global(instance: Self) {
        INSTANCE.set(instance).unwrap();
    }

    pub fn playlist_path(playlist_name: &str) -> PathBuf {
        PathBuf::from(&Config::global().playlists_dir).join(format!("{}.m3u8", playlist_name))
    }

    pub fn merge(mut self, other: OptionalConfig) -> Self {
        if let Some(playlists_dir) = other.playlists_dir {
            self.playlists_dir = playlists_dir;
        }

        if let Some(normal) = other.normal {
            for (k, v) in normal.0 {
                self.normal.0.insert(k, v);
            }
        }

        if let Some(visualizer_gradient) = other.visualizer_gradient {
            self.visualizer_gradient = visualizer_gradient;
        }

        self
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut me: Self = serde_yaml::from_str(std::include_str!("default_config.yaml"))
            .expect("src/default_config.yaml is not valid yaml!");

        me.playlists_dir = dirs::audio_dir()
            .map(|p| p.join("tori"))
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or("playlists".into());

        me
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OptionalConfig {
    pub playlists_dir: Option<String>,
    pub visualizer_gradient: Option<[(u8, u8, u8); 2]>,
    pub normal: Option<Shortcuts>,
}

impl OptionalConfig {
    /// Loads the shortcuts from some path
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        match std::fs::File::open(path) {
            Ok(file) => serde_yaml::from_reader(file)
                .map_err(|e| format!("Couldn't parse your config.yaml. Reason: {}", e).into()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }
}
