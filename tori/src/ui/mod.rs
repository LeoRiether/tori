pub mod eventful_widget;
pub use eventful_widget::*;

pub mod scrollbar;
pub use scrollbar::Scrollbar;

pub mod list;
pub use list::*;

use crate::{
    config::Config,
    events::{Action, Command},
    rect_ops::RectOps,
    state::{
        browse_screen::{BrowseScreen, Focus, SortingMethod},
        Screen, State,
    },
};
use tui::{prelude::*, widgets::Borders};

pub fn ui(state: &mut State, area: Rect, buf: &mut Buffer) {
    let mut l = std::mem::take(&mut state.listeners);
    l.clear();

    state.visualizer.render(area, buf);

    let (screen_area, bottom) = area.split_bottom(2);
    state.now_playing.render(bottom, buf, &mut l);

    match &mut state.screen {
        Screen::None => unreachable!(),
        Screen::BrowseScreen(screen) => browse_screen(screen, screen_area, buf, &mut l),
    }

    if let Some(n) = state.notification.as_ref() {
        n.render(area, buf)
    }
    if let Some(m) = state.modal.as_ref() {
        m.render(area, buf)
    }

    state.listeners = l;
}

/// Draws the screen that allows the user to browse their playlists and songs.
fn browse_screen(
    screen: &mut BrowseScreen,
    area: Rect,
    buf: &mut Buffer,
    l: &mut Vec<Listener<Action>>,
) {
    let (left, right) = area.split_vertically_p(0.15);
    playlists_pane(screen, left, buf, l);
    songs_pane(screen, right, buf, l);
}

/// Draws the pane that shows the user's playlists contained in the browse screen.
fn playlists_pane(
    screen: &mut BrowseScreen,
    area: Rect,
    buf: &mut Buffer,
    l: &mut Vec<Listener<Action>>,
) {
    let title = match &screen.focus {
        Focus::PlaylistsFilter(f) => format!(" /{} ", f.value),
        _ => " playlists ".to_string(),
    };

    let border_style = if matches!(screen.focus, Focus::PlaylistsFilter(_) | Focus::Playlists) {
        Style::default().fg(Color::LightBlue)
    } else {
        Style::default()
    };

    let key = Config::global()
        .keybindings
        .0
        .iter()
        .find(|&(_key, &cmd)| cmd == Command::Add)
        .map(|(key, _)| key.0.as_str())
        .unwrap_or("a");

    let help = format!(
        "You don't have any playlists yet! Press '{}' to add one.",
        key
    );

    let playlists: Vec<_> = screen
        .shown_playlists
        .iter()
        .map(|&i| [format!(" {}", screen.playlists[i])])
        .collect();

    let mut list = List::default()
        .title(title)
        .items(playlists)
        .state(screen.shown_playlists.state.clone())
        .help_message(help)
        .border_style(border_style)
        .borders(Borders::ALL & !Borders::RIGHT)
        .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::Black))
        .click_event(Action::SelectPlaylist);
    list.render(area, buf, l);
    screen.shown_playlists.state = list.get_state();
}

/// Draws the pane that shows the songs of a playlist inside the browse screen
fn songs_pane(
    screen: &mut BrowseScreen,
    area: Rect,
    buf: &mut Buffer,
    l: &mut Vec<Listener<Action>>,
) {
    let sorting = match screen.sorting_method {
        SortingMethod::Index => "",
        SortingMethod::Title => " [↑ Title]",
        SortingMethod::Duration => " [↑ Duration]",
    };

    let title = match &screen.focus {
        Focus::SongsFilter(filter) => format!(" /{}{} ", filter.value, sorting),
        _ => screen
            .selected_playlist()
            .map(|p| format!(" {} ", p))
            .unwrap_or_default(),
    };

    let border_style = if matches!(screen.focus, Focus::SongsFilter(_) | Focus::Songs) {
        Style::default().fg(Color::LightBlue)
    } else {
        Style::default()
    };

    let key = Config::global()
        .keybindings
        .0
        .iter()
        .find(|&(_key, &cmd)| cmd == Command::Add)
        .map(|(key, _)| key.0.as_str())
        .unwrap_or("a");

    let help = format!(
        "You don't have any songs in this playlist yet! Press '{}' to add one.",
        key
    );

    let songlist: Vec<_> = screen
        .shown_songs
        .iter()
        .map(|&i| {
            let song = &screen.songs[i];
            let secs = song.duration.as_secs();
            [
                format!(" {}", song.title),
                format!("{}:{:02}", secs / 60, secs % 60),
            ]
        })
        .collect();

    let mut list = List::default()
        .title(title)
        .items(songlist)
        .state(screen.shown_songs.state.clone())
        .help_message(help)
        .border_style(border_style)
        .borders(Borders::ALL)
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
        .highlight_symbol(" ◇")
        .click_event(Action::SelectSong);
    list.render(area, buf, l);
    screen.shown_songs.state = list.get_state();
}
