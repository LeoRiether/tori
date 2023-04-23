use crate::{
    app::{
        component::{Component, MouseHandler},
        filtered_list::FilteredList,
        App, Mode, MyBackend,
    },
    config::Config,
    error::Result,
    events::Event,
};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEventKind};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use std::io;
use std::result::Result as StdResult;
use tui::{
    layout::{self, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};

// For the horrible_hack_to_get_offset
struct MyListState {
    offset: usize,
    _selected: Option<usize>,
}

fn horrible_hack_to_get_offset(state: &ListState) -> usize {
    // SAFETY: there are some tests that try to check if ListState and MyListState have the same
    // layout, but that's it :)
    unsafe { std::mem::transmute::<&ListState, &MyListState>(state).offset }
}

#[derive(Debug, Default)]
pub struct PlaylistsPane {
    playlists: Vec<String>,
    shown: FilteredList<ListState>,
    filter: String,
}

impl PlaylistsPane {
    pub fn new() -> Result<Self> {
        let mut me = Self::default();
        me.reload_from_dir()?;
        Ok(me)
    }

    pub fn reload_from_dir(&mut self) -> Result<()> {
        let dir = std::fs::read_dir(&Config::global().playlists_dir)
            .map_err(|e| format!("Failed to read playlists directory: {}", e))?;

        use std::fs::DirEntry;
        let extract_playlist_name = |entry: StdResult<DirEntry, io::Error>| {
            entry
                .unwrap()
                .file_name()
                .into_string()
                .unwrap()
                .trim_end_matches(".m3u8")
                .to_string()
        };

        self.playlists = dir.into_iter().map(extract_playlist_name).collect();
        self.playlists.sort();
        self.refresh_shown();
        Ok(())
    }

    fn refresh_shown(&mut self) {
        self.shown.filter(
            &self.playlists,
            |s| {
                self.filter.is_empty()
                    || s.to_lowercase()
                        .contains(&self.filter[1..].trim_end_matches('\n').to_lowercase())
            },
            |i, j| i.cmp(&j),
        );
    }

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

    pub fn select_next(&mut self, app: &mut App) {
        self.shown.select_next();
        app.channel.send(Event::ChangedPlaylist).unwrap();
    }

    pub fn select_prev(&mut self, app: &mut App) {
        self.shown.select_prev();
        app.channel.send(Event::ChangedPlaylist).unwrap();
    }

    pub fn select_index(&mut self, app: &mut App, i: Option<usize>) {
        self.shown.state.select(i);
        app.channel.send(Event::ChangedPlaylist).unwrap();
    }

    pub fn selected_item(&self) -> Option<&str> {
        self.shown
            .selected_item()
            .and_then(|i| self.playlists.get(i))
            .map(|s| s.as_str())
    }

    pub fn open_editor_for_selected(&mut self, app: &mut App) -> Result<()> {
        if let Some(selected) = self.selected_item() {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

            let _lock = app.channel.receiving_crossterm.lock().unwrap();
            io::stdout().execute(LeaveAlternateScreen)?;

            let res = std::process::Command::new(&editor)
                .arg(Config::playlist_path(selected))
                .status()
                .map_err(|err| format!("Failed to execute editor '{}': {}", editor, err));

            io::stdout().execute(EnterAlternateScreen)?;

            res?;
            self.reload_from_dir()?;
            app.terminal.clear()?;
        }
        Ok(())
    }

    fn click(&mut self, app: &mut App, frame: Rect, y: u16) {
        let top = frame
            .inner(&layout::Margin {
                vertical: 1,
                horizontal: 1,
            })
            .top();
        let line = y.saturating_sub(top) as usize;
        let index = line + horrible_hack_to_get_offset(&self.shown.state);
        if index < self.shown.items.len() && Some(index) != self.shown.selected_item() {
            self.select_index(app, Some(index));
        }
    }
}

impl Component for PlaylistsPane {
    type RenderState = bool;

    fn mode(&self) -> Mode {
        if self.filter.is_empty() || self.filter.as_bytes().last() == Some(&b'\n') {
            Mode::Normal
        } else {
            Mode::Insert
        }
    }

    fn render(&mut self, frame: &mut Frame<'_, MyBackend>, chunk: layout::Rect, is_focused: bool) {
        let title = if !self.filter.is_empty() {
            format!(" {} ", self.filter)
        } else {
            " playlists ".into()
        };

        let mut block = Block::default()
            .title(title)
            .borders(Borders::LEFT | Borders::BOTTOM | Borders::TOP)
            .border_type(BorderType::Plain);

        if is_focused {
            block = block.border_style(Style::default().fg(Color::LightBlue));
        }

        let playlists: Vec<_> = self
            .shown
            .items
            .iter()
            .map(|&i| ListItem::new(self.playlists[i].as_str()))
            .collect();

        let widget = List::new(playlists)
            .block(block)
            .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::Black));
        frame.render_stateful_widget(widget, chunk, &mut self.shown.state);
    }

    #[allow(clippy::collapsible_match)]
    #[allow(clippy::single_match)]
    fn handle_event(&mut self, app: &mut App, event: Event) -> Result<()> {
        use crate::command::Command::*;
        use Event::*;
        use KeyCode::*;

        match event {
            Command(cmd) => match cmd {
                SelectNext => self.select_next(app),
                SelectPrev => self.select_prev(app),
                _ => {}
            },
            Terminal(event) => match event {
                crossterm::event::Event::Key(event) => {
                    if self.mode() == Mode::Insert && self.handle_filter_key_event(event)? {
                        self.refresh_shown();
                        app.channel.send(Event::ChangedPlaylist).unwrap();
                        return Ok(());
                    }

                    match event.code {
                        Up => self.select_prev(app),
                        Down => self.select_next(app),
                        Char('/') => self.filter = "/".into(),
                        Esc => {
                            self.filter.clear();
                            self.refresh_shown();
                            app.channel.send(Event::ChangedPlaylist).unwrap();
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }
}

impl MouseHandler for PlaylistsPane {
    fn handle_mouse(
        &mut self,
        app: &mut App,
        chunk: Rect,
        event: crossterm::event::MouseEvent,
    ) -> Result<()> {
        match event.kind {
            MouseEventKind::ScrollUp => self.select_prev(app),
            MouseEventKind::ScrollDown => self.select_next(app),
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left) => {
                self.click(app, chunk, event.row)
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
    fn test_my_list_state_size() {
        assert_eq!(size_of::<MyListState>(), size_of::<ListState>());
    }

    #[test]
    fn test_my_list_state_offset() {
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
        let mut state = ListState::default();
        for _ in 0..10 {
            let offset = thread_rng().gen_range(0..32);
            state.select(Some(offset));

            // Render table in a really small area so it changes the state.offset
            let table = List::new(vec![ListItem::new("Hello World!"); items]);
            table.render(area, &mut buf, &mut state);

            // Assert our hack worked
            assert_eq!(horrible_hack_to_get_offset(&state), offset);
        }
    }
}
