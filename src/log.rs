const DEBUG_FILE: &str = "log";

pub fn __debug_internal(msg: &str) {
    use std::io::Write;
    let f = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(DEBUG_FILE);
    if let Ok(mut f) = f {
        let _ = writeln!(f, "{msg}");
    }
}

pub fn __fatal_internal(msg: &str) -> ! {
    use std::io::Write;
    let _ = writeln!(std::io::stderr(), "\x1b[0;31m[-]\x1b[0m {msg}");
    std::process::exit(1)
}

macro_rules! debug {
    ($($arg:tt)*) => {{
        use crate::log::__debug_internal;
        #[cfg(debug_assertions)]
        __debug_internal(&format!($($arg)*));

    }};
}
pub(crate) use debug;

macro_rules! fatal {
    ($($arg:tt)*) => {{
        use crate::log::__fatal_internal;
        __fatal_internal(&format!($($arg)*));
    }};
}
pub(crate) use fatal;
