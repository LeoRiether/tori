use tui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

/// Widget that draws a scrollbar at the right side of a chunk
#[derive(Debug, Default)]
pub struct Scrollbar {
    /// Line/position of the scrollable component that's currently selected
    pub position: u16,

    /// Total height of the scrollable component
    pub total_height: u16,

    pub style: Style,
}

impl Scrollbar {
    pub fn new(position: u16, total_height: u16) -> Self {
        Self {
            position,
            total_height,
            ..Default::default()
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for Scrollbar {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        self.total_height = std::cmp::max(1, self.total_height);
        let scrollbar_height = (area.height / self.total_height).max(2).min(6);
        let pos = (self.position as f64 / self.total_height as f64 * area.height as f64).round()
            as u16
            + area.top();

        for line in pos..(pos + scrollbar_height).min(area.bottom()) {
            buf.set_string(area.right().saturating_sub(1), line, "█", self.style);
        }
    }
}
