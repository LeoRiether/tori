//! I don't really know where to put these...

use std::time::{Duration, Instant};

use tui::layout::Rect;

////////////////////////////////
//        RectContains        //
////////////////////////////////
pub trait RectContains {
    fn contains(&self, x: u16, y: u16) -> bool;
}

impl RectContains for Rect {
    fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.left() && x <= self.right() && y >= self.top() && y <= self.bottom()
    }
}

/////////////////////////////
//        ClickInfo        //
/////////////////////////////
#[derive(Debug)]
pub struct ClickUpdateSummary {
    pub double_click: bool,
}

#[derive(Debug)]
pub struct ClickInfo {
    pub instant: Instant,
    pub y: u16,
}

impl ClickInfo {
    /// Updates the ClickInfo with another click
    pub fn update(last_click: &mut Option<ClickInfo>, y: u16) -> ClickUpdateSummary {
        let this_click = ClickInfo {
            instant: Instant::now(),
            y,
        };

        let summary = if let Some(s_last_click) = last_click {
            let double_click = s_last_click.instant.elapsed() <= Duration::from_millis(200)
                && this_click.y == s_last_click.y;

            ClickUpdateSummary { double_click }
        } else {
            ClickUpdateSummary {
                double_click: false,
            }
        };

        *last_click = Some(this_click);
        summary
    }
}

/////////////////////////////
//        Clipboard        //
/////////////////////////////
#[cfg(feature = "clip")]
pub fn copy_to_clipboard(text: String) {
    use clipboard::{ClipboardContext, ClipboardProvider};
    let mut ctx: ClipboardContext = ClipboardContext::new().unwrap();
    ctx.set_contents(text).unwrap();
}

#[cfg(not(feature = "clip"))]
pub fn copy_to_clipboard(_text: String) {}
