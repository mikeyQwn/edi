const DEBUG_FILE: &str = "log";

pub fn __debug_internal(msg: String) {
    use std::io::Write;
    let f = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(DEBUG_FILE);
    if let Ok(mut f) = f {
        let _ = writeln!(f, "{}", msg);
    }
}

macro_rules! debug {
    ($($arg:tt)*) => {{
        use crate::log::__debug_internal;
        #[cfg(debug_assertions)]
        __debug_internal(format!($($arg)*));

    }};
}
pub(crate) use debug;
