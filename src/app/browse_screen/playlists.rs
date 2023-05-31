use crate::{
    app::{
        component::{Component, MouseHandler},
        filtered_list::FilteredList,
        App, Mode, MyBackend,
    },
    command::Command,
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
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

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
        let index = line + self.shown.state.offset();
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

        if !self.playlists.is_empty() {
            // Render playlists list
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
                "You don't have any playlists yet! Press '{}' to add one.",
                key
            ))
            .wrap(Wrap { trim: true })
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(widget, chunk);
        }
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
                Search => self.filter = "/".into(),
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

