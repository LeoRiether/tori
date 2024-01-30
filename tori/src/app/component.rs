use super::App;
use crate::{error::Result, events};
use std::io;
use tui::{backend::CrosstermBackend, layout::Rect, Frame};

pub(crate) type MyBackend = CrosstermBackend<io::Stdout>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(i8)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

pub trait Component {
    type RenderState;

    fn mode(&self) -> Mode;
    fn render(
        &mut self,
        frame: &mut Frame,
        chunk: Rect,
        render_state: Self::RenderState,
    );
    fn handle_event(&mut self, app: &mut App, event: events::Event) -> Result<()>;
}

pub trait MouseHandler {
    fn handle_mouse(
        &mut self,
        app: &mut App,
        chunk: Rect,
        event: crossterm::event::MouseEvent,
    ) -> Result<()>;
}
