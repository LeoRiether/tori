use crate::{app::filtered_list::FilteredList, config::Config, error::Result, m3u, input::Input};
use std::{io, result::Result as StdResult};
use tui::widgets::TableState;

#[derive(Debug, Default)]
pub struct BrowseScreen {
    pub focus: Focus,
    pub playlists: Vec<String>,
    pub shown_playlists: FilteredList<TableState>,

    pub songs: Vec<m3u::Song>,
    pub shown_songs: FilteredList<TableState>,
    pub sorting_method: SortingMethod,
}

#[derive(Debug, Default, Clone)]
#[repr(i8)]
pub enum Focus {
    #[default]
    Playlists,
    Songs,
    PlaylistsFilter(Input),
    SongsFilter(Input),
}

#[derive(Debug, Default, Clone, Copy)]
pub enum SortingMethod {
    #[default]
    Index,
    Title,
    Duration,
}

impl SortingMethod {
    pub fn next(&self) -> Self {
        use SortingMethod::*;
        match self {
            Index => Title,
            Title => Duration,
            Duration => Index,
        }
    }
}

impl BrowseScreen {
    pub fn new() -> Result<Self> {
        let mut me = Self::default();
        me.refresh_playlists()?;
        Ok(me)
    }

    pub fn next_sorting_mode(&mut self) {
        self.sorting_method = self.sorting_method.next();
    }

    /////////////////////////////////////
    //        Refresh playlists        //
    /////////////////////////////////////
    pub fn refresh_playlists(&mut self) -> Result<()> {
        self.load_playlists()?;
        self.filter_playlists();
        self.update_selected_playlist();
        self.refresh_songs()?;
        Ok(())
    }

    fn load_playlists(&mut self) -> Result<()> {
        let dir = std::fs::read_dir(&Config::global().playlists_dir)
            .map_err(|e| format!("Failed to read playlists directory: {}", e))?;

        use std::fs::DirEntry;
        let extract_playlist_name = |entry: StdResult<DirEntry, io::Error>| {
            Ok(entry
                .unwrap()
                .file_name()
                .into_string()
                .map_err(|filename| format!("File '{:?}' has invalid UTF-8", filename))?
                .trim_end_matches(".m3u8")
                .to_string())
        };

        self.playlists = dir
            .into_iter()
            .map(extract_playlist_name)
            .collect::<Result<_>>()?;
        self.playlists.sort();
        Ok(())
    }

    fn filter_playlists(&mut self) {
        match &self.focus {
            Focus::PlaylistsFilter(filter) if !filter.value.is_empty() => {
                let filter = filter.value.trim_end_matches('\n').to_lowercase();
                self.shown_playlists.filter(
                    &self.playlists,
                    |s| s.to_lowercase().contains(&filter),
                    |i, j| i.cmp(&j),
                )
            }
            _ => self
                .shown_playlists
                .filter(&self.playlists, |_| true, |i, j| i.cmp(&j)),
        }
    }

    fn update_selected_playlist(&mut self) {
        // Try to reuse previous state
        let state = self.shown_playlists.state.clone();
        if matches!(state.selected(), Some(i) if i < self.playlists.len()) {
            self.shown_playlists.state = state;
        } else if self.shown_playlists.items.is_empty() {
            self.shown_playlists.state.select(None);
        } else {
            self.shown_playlists.state.select(Some(0));
        }
    }

    /////////////////////////////////
    //        Refresh songs        //
    /////////////////////////////////
    pub fn refresh_songs(&mut self) -> Result<()> {
        self.load_songs()?;
        self.filter_songs();
        self.update_selected_song();
        Ok(())
    }

    fn load_songs(&mut self) -> Result<()> {
        let playlist_name = match self.selected_playlist() {
            Some(name) => name,
            None => {
                self.songs = Vec::new();
                self.shown_songs = Default::default();
                return Ok(());
            }
        };

        let path = Config::playlist_path(playlist_name);
        let file = std::fs::File::open(&path)
            .map_err(|_| format!("Couldn't open playlist file {}", path.display()))?;

        self.songs = m3u::Parser::from_reader(file).all_songs()?;
        Ok(())
    }

    fn filter_songs(&mut self) {
        match &self.focus {
            Focus::SongsFilter(filter) if !filter.value.is_empty() => {
                let filter = filter.value.trim_end_matches('\n').to_lowercase();
                let pred = |s: &m3u::Song| {
                    s.title.to_lowercase().contains(&filter)
                        || s.path.to_lowercase().contains(&filter)
                };
                let cmp = |i, j| compare_songs(i, j, &self.songs, self.sorting_method);
                self.shown_songs.filter(&self.songs, pred, cmp)
            }
            _ => self
                .shown_songs
                .filter(&self.songs, |_| true, |i, j| i.cmp(&j)),
        }
    }

    fn update_selected_song(&mut self) {
        let state = self.shown_songs.state.clone();
        if matches!(state.selected(), Some(i) if i < self.songs.len()) {
            self.shown_songs.state = state;
        } else if self.shown_songs.items.is_empty() {
            self.shown_songs.state.select(None);
        } else {
            self.shown_songs.state.select(Some(0));
        }
    }

    /////////////////////////////
    //        Selectors        //
    /////////////////////////////
    pub fn select_next(&mut self) -> Result<()> {
        match self.focus {
            Focus::Playlists => {
                self.shown_playlists.select_next();
                self.refresh_songs()?;
            }
            Focus::Songs => self.shown_songs.select_next(),
            Focus::PlaylistsFilter(_) | Focus::SongsFilter(_) => {}
        }
        Ok(())
    }

    pub fn select_prev(&mut self) -> Result<()> {
        match self.focus {
            Focus::Playlists => {
                self.shown_playlists.select_prev();
                self.refresh_songs()?;
            }
            Focus::Songs => self.shown_songs.select_prev(),
            Focus::PlaylistsFilter(_) | Focus::SongsFilter(_) => {}
        }
        Ok(())
    }

    pub fn selected_playlist(&self) -> Option<&str> {
        self.shown_playlists
            .selected_item()
            .and_then(|i| self.playlists.get(i))
            .map(|s| s.as_str())
    }

    pub fn selected_playlist_index(&self) -> Option<usize> {
        self.shown_playlists.selected_item()
    }

    pub fn selected_song(&self) -> Option<&m3u::Song> {
        self.shown_songs
            .selected_item()
            .and_then(|i| self.songs.get(i))
    }

    pub fn selected_song_index(&self) -> Option<usize> {
        self.shown_songs.selected_item()
    }
}

fn compare_songs(
    i: usize,
    j: usize,
    songs: &[m3u::Song],
    method: SortingMethod,
) -> std::cmp::Ordering {
    match method {
        SortingMethod::Index => i.cmp(&j),
        SortingMethod::Title => songs[i].title.cmp(&songs[j].title),
        SortingMethod::Duration => songs[i].duration.cmp(&songs[j].duration),
    }
}
