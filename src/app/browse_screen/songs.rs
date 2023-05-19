use std::borrow::Cow;

use std::path::Path;

use crate::app::component::MouseHandler;
use crate::command::Command;
use crate::error::Result;
use crate::events::Event;
use crate::util::ClickInfo;
use crate::widgets::Scrollbar;
use crate::{
    app::{component::Component, filtered_list::FilteredList, App, Mode, MyBackend},
    config::Config,
};
use crate::{m3u, util};

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEventKind};
use tui::layout::Rect;
use tui::widgets::{Paragraph, Wrap};
use tui::{
    layout::{self, Constraint},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Row, Table, TableState},
    Frame,
};

struct MyTableState {
    offset: usize,
    _selected: Option<usize>,
}

fn horrible_hack_to_get_offset(state: &TableState) -> usize {
    // SAFETY: there are some tests that try to check if ListState and MyListState have the same
    // layout, but that's it :)
    unsafe { std::mem::transmute::<&TableState, &MyTableState>(state).offset }
}

/////////////////////////////////
//        SortingMethod        //
/////////////////////////////////
#[derive(Debug, Default, Clone, Copy)]
enum SortingMethod {
    #[default]
    /// identity permutation
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

//////////////////////////////////////
//        MousePressLocation        //
//////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq)]
enum MousePressLocation {
    List,
    Scrollbar,
}

/////////////////////////////
//        SongsPane        //
/////////////////////////////
/// Displays the list of songs of a given playlist
#[derive(Debug, Default)]
pub struct SongsPane<'t> {
    /// Generally the name of the playlist
    title: Cow<'t, str>,
    songs: Vec<m3u::Song>,
    shown: FilteredList<TableState>,
    sorting_method: SortingMethod,
    filter: String,
    last_click: Option<ClickInfo>,
    mouse_press_location: Option<MousePressLocation>,
}

impl<'t> SongsPane<'t> {
    pub fn new() -> Self {
        Self {
            title: " songs ".into(),
            ..Default::default()
        }
    }

    pub fn state(&self) -> TableState {
        self.shown.state.clone()
    }
    pub fn set_state(&mut self, state: TableState) {
        self.shown.state = state;
    }

    pub fn update_from_playlist_pane(
        &mut self,
        playlists: &super::playlists::PlaylistsPane,
    ) -> Result<()> {
        match playlists.selected_item() {
            Some(playlist) => self.update_from_playlist_named(playlist),
            None => {
                *self = SongsPane::new();
                Ok(())
            }
        }
    }

    pub fn update_from_playlist_named(&mut self, name: &str) -> Result<()> {
        self.update_from_playlist(Config::playlist_path(name))
    }

    pub fn update_from_playlist(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::open(&path)
            .map_err(|_| format!("Couldn't open playlist file {}", path.as_ref().display()))?;

        let title = Cow::Owned(
            path.as_ref()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        );

        let songs = m3u::Parser::from_reader(file).all_songs()?;
        let state = self.state();

        // Update stuff
        self.title = title;
        self.songs = songs;
        self.filter.clear();
        self.refresh_shown();

        // Try to reuse previous state
        if matches!(state.selected(), Some(i) if i < self.songs.len()) {
            self.set_state(state);
        } else if self.shown.items.is_empty() {
            self.select_index(None);
        } else {
            self.select_index(Some(0));
        }

        Ok(())
    }

    fn refresh_shown(&mut self) {
        let pred = |s: &m3u::Song| {
            self.filter.is_empty()
                || s.title
                    .to_lowercase()
                    .contains(&self.filter[1..].trim_end_matches('\n').to_lowercase())
                || s.path
                    .to_lowercase()
                    .contains(&self.filter[1..].trim_end_matches('\n').to_lowercase())
        };
        let comparison = |i, j| compare_songs(i, j, &self.songs, self.sorting_method);
        self.shown.filter(&self.songs, pred, comparison);
    }

    fn next_sorting_method(&mut self) {
        self.sorting_method = self.sorting_method.next();
    }

    #[allow(clippy::single_match)]
    fn handle_terminal_event(
        &mut self,
        app: &mut App,
        event: crossterm::event::Event,
    ) -> Result<()> {
        use KeyCode::*;

        match event {
            crossterm::event::Event::Key(event) => {
                if self.mode() == Mode::Insert && self.handle_filter_key_event(event)? {
                    self.refresh_shown();
                    return Ok(());
                }

                match event.code {
                    Enter => self.play_selected(app)?,
                    Esc => {
                        self.filter.clear();
                        self.refresh_shown();
                    }
                    // Go to the top, kind of like in vim
                    Char('g') if self.mode() == Mode::Normal => {
                        if !self.shown.items.is_empty() {
                            self.shown.state.select(Some(0));
                        }
                    }
                    // Go to the bottom, also like in vim
                    Char('G') if self.mode() == Mode::Normal => {
                        if !self.shown.items.is_empty() {
                            self.shown.state.select(Some(self.shown.items.len() - 1));
                        }
                    }
                    Up => self.select_prev(),
                    Down => self.select_next(),
                    Char('/') => self.filter = "/".into(),
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_command(&mut self, app: &mut App, cmd: crate::command::Command) -> Result<()> {
        use crate::command::Command::*;

        match cmd {
            SelectNext => self.select_next(),
            SelectPrev => self.select_prev(),
            QueueSong => {
                if let Some(song) = self.selected_item() {
                    app.mpv.playlist_load_files(&[(
                        &song.path,
                        libmpv::FileState::AppendPlay,
                        None,
                    )])?;
                }
            }
            QueueShown => {
                let entries: Vec<_> = self
                    .shown
                    .items
                    .iter()
                    .map(|&i| {
                        (
                            self.songs[i].path.as_str(),
                            libmpv::FileState::AppendPlay,
                            None::<&str>,
                        )
                    })
                    .collect();
                app.mpv.playlist_load_files(&entries)?;
            }
            Shuffle => {
                app.mpv.command("playlist-shuffle", &[])?;
            }
            OpenInBrowser => {
                if let Some(song) = self.selected_item() {
                    // TODO: reconsider if I really need a library to write this one line
                    webbrowser::open(&song.path)?;
                }
            }
            CopyUrl => {
                if let Some(song) = self.selected_item() {
                    util::copy_to_clipboard(song.path.clone());
                    #[cfg(feature = "clip")]
                    app.notify_info(format!("Copied {} to the clipboard", song.path));
                    #[cfg(not(feature = "clip"))]
                    app.notify_info("Clipboard support is disabled for this build. You can enable it by building with '--features clip'");
                }
            }
            CopyTitle => {
                if let Some(song) = self.selected_item() {
                    util::copy_to_clipboard(song.title.clone());
                    #[cfg(feature = "clip")]
                    app.notify_info(format!("Copied {} to the clipboard", song.title));
                    #[cfg(not(feature = "clip"))]
                    app.notify_info("Clipboard support is disabled for this build. You can enable it by building with '--features clip'");
                }
            }
            SwapSongUp if self.filter.is_empty() => match self.selected_index() {
                Some(i) if i >= 1 => {
                    m3u::playlist_management::swap_song(&self.title, i - 1)?;
                    self.songs.swap(i - 1, i);
                    self.select_prev();
                }
                _ => {}
            },
            SwapSongDown if self.filter.is_empty() => {
                if let Some(i) = self.selected_index() {
                    m3u::playlist_management::swap_song(&self.title, i)?;
                    self.songs.swap(i, i + 1);
                    self.select_next();
                }
            }
            NextSortingMode => {
                self.next_sorting_method();
                self.refresh_shown();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles a key event when the filter is active.
    pub fn handle_filter_key_event(&mut self, event: KeyEvent) -> Result<bool> {
        match event.code {
            KeyCode::Char(c) => {
                self.filter.push(c);
                Ok(true)
            }
            KeyCode::Backspace => {
                self.filter.pop();
                Ok(true)
            }
            KeyCode::Esc => {
                self.filter.clear();
                Ok(true)
            }
            KeyCode::Enter => {
                self.filter.push('\n');
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Handle a click on the song component
    pub fn click(
        &mut self,
        app: &mut App,
        chunk: Rect,
        (x, y): (u16, u16),
        kind: MouseEventKind,
    ) -> Result<()> {
        match kind {
            MouseEventKind::Up(MouseButton::Left) => {
                self.mouse_press_location = None;
            }
            // If the mouse press (MouseEventKind::Down event) was done on the scrollbar,
            // any drag events will still be handled by the scrollbar, even if the mouse
            // is moved outside of the scrollbar.
            MouseEventKind::Drag(MouseButton::Left)
                if self.mouse_press_location == Some(MousePressLocation::Scrollbar) =>
            {
                self.click_scrollbar(app, chunk, (x, y), kind)?;
            }
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left) => {
                if x + 1 == chunk.right() {
                    // Clicked on the scrollbar
                    self.click_scrollbar(app, chunk, (x, y), kind)?;
                } else {
                    // Clicked on the song list
                    self.click_list(app, chunk, (x, y), kind)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle a click on the scrollbar
    fn click_scrollbar(
        &mut self,
        _app: &mut App,
        chunk: Rect,
        (_x, y): (u16, u16),
        kind: MouseEventKind,
    ) -> Result<()> {
        if let MouseEventKind::Down(MouseButton::Left) = kind {
            self.mouse_press_location = Some(MousePressLocation::Scrollbar);
        }

        let perc = (y as f64 - chunk.top() as f64 - 1.0) / (chunk.height - 2) as f64;
        let len = self.shown.items.len().saturating_sub(1);
        self.select_index(Some(((perc * len as f64) as usize).max(0).min(len)));
        Ok(())
    }

    /// Handle a click on the song list
    fn click_list(
        &mut self,
        app: &mut App,
        chunk: Rect,
        (_x, y): (u16, u16),
        kind: MouseEventKind,
    ) -> Result<()> {
        if let MouseEventKind::Down(MouseButton::Left) = kind {
            self.mouse_press_location = Some(MousePressLocation::List);
        }

        // Compute clicked item
        let top = chunk
            .inner(&layout::Margin {
                vertical: 1,
                horizontal: 1,
            })
            .top();
        let line = y.saturating_sub(top) as usize;
        let index = line + horrible_hack_to_get_offset(&self.shown.state);

        // Update self.last_click with current click
        let click_summary = ClickInfo::update(&mut self.last_click, y);

        // User clicked outside the list
        if index >= self.shown.items.len() {
            return Ok(());
        }

        // Select song
        self.select_index(Some(index));

        // If it's a double click, play this selected song
        if click_summary.double_click && matches!(kind, MouseEventKind::Down(MouseButton::Left)) {
            self.play_selected(app)?;
        }
        Ok(())
    }

    pub fn play_selected(&self, app: &mut App) -> Result<()> {
        if let Some(song) = self.selected_item() {
            app.mpv
                .playlist_load_files(&[(&song.path, libmpv::FileState::Replace, None)])?;
        }
        Ok(())
    }

    pub fn select_next(&mut self) {
        self.shown.select_next();
    }

    pub fn select_prev(&mut self) {
        self.shown.select_prev();
    }

    pub fn select_index(&mut self, i: Option<usize>) {
        self.shown.state.select(i);
    }

    pub fn selected_item(&self) -> Option<&m3u::Song> {
        self.shown.selected_item().and_then(|i| self.songs.get(i))
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.shown.selected_item()
    }
}

impl<'t> Component for SongsPane<'t> {
    type RenderState = bool;

    fn mode(&self) -> Mode {
        if self.filter.is_empty() || self.filter.as_bytes().last() == Some(&b'\n') {
            Mode::Normal
        } else {
            Mode::Insert
        }
    }

    fn render(&mut self, frame: &mut Frame<'_, MyBackend>, chunk: layout::Rect, is_focused: bool) {
        let sorting = match self.sorting_method {
            SortingMethod::Index => "",
            SortingMethod::Title => " [↑ Title]",
            SortingMethod::Duration => " [↑ Duration]",
        };

        let title = if !self.filter.is_empty() {
            format!(" {}{} ", self.filter, sorting)
        } else {
            format!(" {}{} ", self.title, sorting)
        };

        let border_style = if is_focused {
            Style::default().fg(Color::LightBlue)
        } else {
            Style::default()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(border_style);

        if !self.songs.is_empty() {
            // Render songlist
            let songlist: Vec<_> = self
                .shown
                .items
                .iter()
                .map(|&i| &self.songs[i])
                .map(|song| {
                    Row::new(vec![
                        format!(" {}", song.title),
                        format!(
                            "{}:{:02}",
                            song.duration.as_secs() / 60,
                            song.duration.as_secs() % 60
                        ),
                    ])
                })
                .collect();
            let songlist_len = songlist.len();

            // Render table
            let widths = &[Constraint::Length(chunk.width - 11), Constraint::Length(10)];
            let widget = Table::new(songlist)
                .block(block)
                .widths(widths)
                .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
                .highlight_symbol(" ◇");
            frame.render_stateful_widget(widget, chunk, &mut self.shown.state);

            if self.shown.items.len() > chunk.height as usize - 2 {
                // Render scrollbar
                let scrollbar = Scrollbar::new(
                    self.shown.state.selected().unwrap_or(0) as u16,
                    songlist_len as u16,
                )
                .with_style(border_style);
                frame.render_widget(
                    scrollbar,
                    chunk.inner(&tui::layout::Margin {
                        vertical: 1,
                        horizontal: 0,
                    }),
                );
            }
        } else {
            // Help message
            let key = Config::global()
                .keybindings
                .0
                .iter()
                .find(|&(_key, &cmd)| cmd == Command::Add)
                .map(|(key, _)| key.0.as_str())
                .unwrap_or("a");

            let widget = Paragraph::new(format!(
                "You don't have any songs in this playlist yet! Press '{}' to add one.",
                key
            ))
            .wrap(Wrap { trim: true })
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(widget, chunk);
        }
    }

    fn handle_event(&mut self, app: &mut App, event: Event) -> Result<()> {
        use Event::*;

        match event {
            Command(cmd) => self.handle_command(app, cmd)?,
            Terminal(event) => self.handle_terminal_event(app, event)?,
            SongAdded {
                playlist: _,
                song: _,
            } => {
                // scroll to the bottom
                if !self.shown.items.is_empty() {
                    self.shown.state.select(Some(self.shown.items.len() - 1));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl<'a> MouseHandler for SongsPane<'a> {
    fn handle_mouse(
        &mut self,
        app: &mut App,
        chunk: Rect,
        event: crossterm::event::MouseEvent,
    ) -> Result<()> {
        match event.kind {
            MouseEventKind::ScrollUp => self.select_prev(),
            MouseEventKind::ScrollDown => self.select_next(),
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left) => {
                self.click(app, chunk, (event.column, event.row), event.kind)?
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};
    use std::mem::size_of;
    use tui::buffer::{Buffer, Cell};
    use tui::widgets::StatefulWidget;

    #[test]
    fn test_my_table_state_size() {
        assert_eq!(size_of::<MyTableState>(), size_of::<TableState>());
    }

    #[test]
    fn test_my_table_state_offset() {
        let area = Rect {
            x: 0,
            width: 64,
            y: 0,
            height: 1, // ensures offset == selected
        };
        let mut buf = Buffer {
            area,
            content: vec![Cell::default(); 64],
        };

        let items = 32;
        let mut state = TableState::default();
        for _ in 0..10 {
            let offset = thread_rng().gen_range(0..32);
            state.select(Some(offset));

            // Render table in a really small area so it changes the state.offset
            let table = Table::new(vec![Row::new(vec!["Hello World!"]); items]);
            table.render(area, &mut buf, &mut state);

            // Assert our hack worked
            assert_eq!(horrible_hack_to_get_offset(&state), offset);
        }
    }
}
