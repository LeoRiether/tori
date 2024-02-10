use std::cmp::min;
use tui::layout::Rect;

pub trait RectOps {
    fn contains(&self, x: u16, y: u16) -> bool;
    fn split_top(&self, n: u16) -> (Rect, Rect);
    fn split_bottom(&self, n: u16) -> (Rect, Rect);
    fn split_left(&self, n: u16) -> (Rect, Rect);
    fn split_right(&self, n: u16) -> (Rect, Rect);
    fn split_horizontally_p(&self, p: f64) -> (Rect, Rect);
    fn split_vertically_p(&self, p: f64) -> (Rect, Rect);
}

impl RectOps for Rect {
    fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.left() && x <= self.right() && y >= self.top() && y <= self.bottom()
    }

    fn split_top(&self, n: u16) -> (Rect, Rect) {
        let top = Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: min(n, self.height),
        };
        let bottom = Rect {
            x: self.x,
            y: self.y.saturating_add(n),
            width: self.width,
            height: self.height.saturating_sub(n),
        };
        (top, bottom)
    }

    fn split_bottom(&self, n: u16) -> (Rect, Rect) {
        let top = Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height.saturating_sub(n),
        };
        let bottom = Rect {
            x: self.x,
            y: self.y.saturating_add(self.height.saturating_sub(n)),
            width: self.width,
            height: min(n, self.height),
        };
        (top, bottom)
    }

    fn split_left(&self, n: u16) -> (Rect, Rect) {
        let left = Rect {
            x: self.x,
            y: self.y,
            width: min(n, self.width),
            height: self.height,
        };
        let right = Rect {
            x: self.x.saturating_add(n),
            y: self.y,
            width: self.width.saturating_sub(n),
            height: self.height,
        };
        (left, right)
    }

    fn split_right(&self, n: u16) -> (Rect, Rect) {
        let left = Rect {
            x: self.x,
            y: self.y,
            width: self.width.saturating_sub(n),
            height: self.height,
        };
        let right = Rect {
            x: self.x.saturating_add(self.width.saturating_sub(n)),
            y: self.y,
            width: min(n, self.width),
            height: self.height,
        };
        (left, right)
    }

    // Split the rect horizontally at the given percentage
    fn split_horizontally_p(&self, p: f64) -> (Rect, Rect) {
        let n = (self.height as f64 * p).round() as u16;
        self.split_top(n)
    }

    // Split the rect vertically at the given percentage
    fn split_vertically_p(&self, p: f64) -> (Rect, Rect) {
        let n = (self.width as f64 * p).round() as u16;
        self.split_left(n)
    }
}
