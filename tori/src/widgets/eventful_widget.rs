use crossterm::event::Event as TermEvent;
use tui::prelude::*;

/// Listener signals that we should emit a message of type `M` when `event` occurs.
pub struct Listener<M> {
    pub event: Event,
    pub emitter: Box<dyn Fn(TermEvent) -> M>,
}

pub fn on<M>(event: Event, emitter: impl Fn(TermEvent) -> M + 'static) -> Listener<M> {
    Listener {
        event,
        emitter: Box::new(emitter),
    }
}

/// An event a widget can receive
pub enum Event {
    Click(Rect),
    _Drag,
    _MouseUp,
}

/// A widget that registers event listeners
pub trait EventfulWidget<M> {
    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Vec<Listener<M>>;
}
