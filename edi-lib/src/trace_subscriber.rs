use std::io;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;
use std::{fs::File, io::Write};

use crate::trace::{Event, Level, Subscriber};

#[derive(Debug)]
pub struct FileLogSubscriber {
    debug_file: Mutex<File>,
}

impl FileLogSubscriber {
    pub fn new(debug_file: impl AsRef<Path>) -> io::Result<Self> {
        let f = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(debug_file)?;

        Ok(Self {
            debug_file: Mutex::new(f),
        })
    }

    fn debug(&self, event: &Event) -> io::Result<()> {
        let Ok(mut file) = self.debug_file.lock() else {
            return Ok(());
        };

        writeln!(
            file,
            "[-] {} [{}] {}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("system clock should not run backwards")
                .as_secs(),
            event.spans_to_string(),
            event.message,
        )?;

        Ok(())
    }

    fn fatal(&self, event: &Event) -> io::Result<()> {
        let msg = event.message.as_ref();
        writeln!(std::io::stderr(), "\x1b[0;31m[-]\x1b[0m {msg}")?;
        Ok(())
    }
}

impl Subscriber for FileLogSubscriber {
    fn enabled(&self, level: Level) -> bool {
        matches!(level, Level::Debug | Level::Fatal)
    }

    fn receive_event(&self, event: Event) {
        let _ = match event.level {
            Level::Debug => self.debug(&event),
            Level::Fatal => self.fatal(&event),
            other => todo!("other levels are not yet implemented in log: {:?}", other),
        };
    }
}
