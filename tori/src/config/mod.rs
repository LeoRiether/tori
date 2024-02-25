use crate::{color::Color, error::Result};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};

pub mod shortcuts;
use shortcuts::Shortcuts;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub playlists_dir: String,
    pub visualizer_gradient: Vec<Color>,
    pub keybindings: Shortcuts,
    pub mpv_ao: Option<String>,
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

        if let Some(keybindings) = other.keybindings {
            for (k, v) in keybindings.0 {
                self.keybindings.0.insert(k, v);
            }
        }

        if let Some(mut visualizer_gradient) = other.visualizer_gradient {
            for color in &mut visualizer_gradient {
                color.precalculate_as_rgb();
            }
            self.visualizer_gradient = visualizer_gradient;
        }

        self.mpv_ao = other.mpv_ao;

        self
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut me: Self = serde_yaml::from_str(std::include_str!("../default_config.yaml"))
            .expect("src/default_config.yaml is not valid yaml!");

        let audio_dir = dirs::audio_dir().filter(|p| p.exists());
        let music_dir = dirs::home_dir()
            .map(|p| p.join("Music"))
            .filter(|p| p.exists());

        me.playlists_dir = audio_dir
            .or(music_dir)
            .map(|p| p.join("tori"))
            .and_then(|p| p.to_str().map(str::to_string))
            .unwrap_or("playlists".into());

        me
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OptionalConfig {
    pub playlists_dir: Option<String>,
    pub visualizer_gradient: Option<Vec<Color>>,
    pub keybindings: Option<Shortcuts>,
    pub mpv_ao: Option<String>,
}

impl OptionalConfig {
    /// Loads the shortcuts from some path
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        match std::fs::File::open(path) {
            Ok(file) => serde_yaml::from_reader(file).map_err(|e| {
                format!("Couldn't parse your tori.yaml config file. Reason: {}", e).into()
            }),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }
}
