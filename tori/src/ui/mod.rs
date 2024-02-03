use crate::{
    events::Event,
    state::{Screen, State},
    widgets::{eventful_widget::Listener, EventfulWidget},
};
use tui::prelude::*;

pub fn ui(state: &mut State, area: Rect, buf: &mut Buffer) -> Vec<Listener<Event>> {
    let mut l = Vec::new();

    state.visualizer.render(area, buf);
    l.append(&mut state.now_playing.render(area, buf));

    match &state.screen {
        Screen::BrowseScreen(_screen) => {}
    }

    if let Some(n) = state.notification.as_ref() {
        n.render(area, buf)
    }
    if let Some(m) = state.modal.as_ref() {
        m.render(area, buf)
    }

    l
}
