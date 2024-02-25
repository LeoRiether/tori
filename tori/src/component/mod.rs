//! Components are self-contained parts of the application that maintain their own state and can
//! render themselves to the screen.

pub mod now_playing;
pub use now_playing::*;

pub mod visualizer;
pub use visualizer::*;

pub mod notification;
pub use notification::*;
