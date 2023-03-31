use std::io::Write;

#[macro_export]
macro_rules! log {
    ($what:expr) => {
        $crate::dbglog::write(format!(concat!(stringify!($what), " = {:?}"), $what).as_str());
    };
}

pub fn write(s: &str) {
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("dbg.log")
        .unwrap();
    f.write_all(s.as_bytes()).unwrap();
    f.write_all(b"\n").unwrap();
}
