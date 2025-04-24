use std::io::Write;
use std::sync::OnceLock;
use std::time::SystemTime;

const DEBUG_FILE: &str = "log";
static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

pub fn set_debug(value: bool) -> bool {
    DEBUG_ENABLED.set(value).is_ok()
}

/// # Panics
///
/// Panics when system clock runs backward
pub fn __debug_internal(msg: &str) {
    if !matches!(DEBUG_ENABLED.get(), Some(true)) {
        return;
    }

    let f = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(DEBUG_FILE);

    if let Ok(mut f) = f {
        let _ = writeln!(
            f,
            "[-] {:?} {msg}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("system clock should not run backwards")
                .as_secs()
        );
    }
}

pub fn __fatal_internal(msg: &str) -> ! {
    let _ = writeln!(std::io::stderr(), "\x1b[0;31m[-]\x1b[0m {msg}");
    std::process::exit(1)
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        use $crate::log::__debug_internal;
        __debug_internal(&format!($($arg)*));

    }};
}
pub(crate) use debug;

#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {{
        use $crate::log::__fatal_internal;
        __fatal_internal(&format!($($arg)*));
    }};
}
