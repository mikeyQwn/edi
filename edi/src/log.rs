use std::io::Write;
use std::sync::OnceLock;
use std::time::SystemTime;

const DEBUG_FILE: &str = "log";
static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

use crate::trace::{Event, Level, Subscriber};

pub fn set_debug(value: bool) -> bool {
    DEBUG_ENABLED.set(value).is_ok()
}

fn __debug_internal(event: &Event) {
    if !matches!(DEBUG_ENABLED.get(), Some(true)) {
        return;
    }

    let f = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(DEBUG_FILE);

    let span_len = event.spans.len();

    if let Ok(mut f) = f {
        let _ = writeln!(
            f,
            "[-] {} [{}] {}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("system clock should not run backwards")
                .as_secs(),
            event
                .spans
                .iter()
                .enumerate()
                .fold(String::new(), |mut acc, (i, span)| {
                    acc.push_str(span.name);
                    if i != span_len.saturating_sub(1) {
                        acc.push_str("::");
                    }
                    acc
                }),
            event.message
        );
    }
}

fn __fatal_internal(msg: &str) {
    let _ = writeln!(std::io::stderr(), "\x1b[0;31m[-]\x1b[0m {msg}");
}

struct LogSubscriber;

impl Subscriber for LogSubscriber {
    fn enabled(&self, level: Level) -> bool {
        matches!(level, Level::Debug | Level::Fatal)
    }

    fn receive_event(&self, event: Event) {
        match event.level {
            Level::Debug => __debug_internal(&event),
            Level::Fatal => __fatal_internal(&event.message),
            other => todo!("other levels are not yet implemented in log: {:?}", other),
        }
    }
}

/// # Errors
///
/// Returns an error when a subscriber is already set
///
pub fn init() -> Result<(), ()> {
    let sub = LogSubscriber;
    crate::trace::set_subscriber(sub)
}
