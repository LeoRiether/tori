//! I don't really know where to put these...

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
