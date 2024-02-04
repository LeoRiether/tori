use crossterm::event::Event as TermEvent;
use tui::prelude::*;

/// Listener signals that we should emit a message of type `M` when `event` occurs.
pub struct Listener<M> {
    pub event: UIEvent,
    pub emitter: Box<dyn Fn(TermEvent) -> M>,
}

pub fn on<M>(event: UIEvent, emitter: impl Fn(TermEvent) -> M + 'static) -> Listener<M> {
    Listener {
        event,
        emitter: Box::new(emitter),
    }
}

/// An event a widget can receive
pub enum UIEvent {
    Click(Rect),
}

/// A widget that registers event listeners
pub trait EventfulWidget<M> {
    fn render(&mut self, area: Rect, buf: &mut Buffer, listeners: &mut Vec<Listener<M>>);
}
