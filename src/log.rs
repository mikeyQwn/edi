use std::io::Write;
use std::sync::OnceLock;
use std::time::SystemTime;

const DEBUG_FILE: &str = "log";
static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

use crate::trace::{Event, Level, Subscriber};

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

struct LogSubscriber;

impl Subscriber for LogSubscriber {
    fn enabled(&self, level: Level) -> bool {
        let _ = level;
        true
    }

    fn receive_event(&self, event: Event) {
        match event.level {
            Level::Debug => __debug_internal(&event.message),
            Level::Fatal => __fatal_internal(&event.message),
            other => todo!("other levels are not yet implemented in log: {:?}", other),
        }
    }
}

pub fn init() -> Result<(), ()> {
    let sub = LogSubscriber;
    crate::trace::set_subscriber(sub)
}
