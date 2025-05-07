use std::{borrow::Cow, sync::OnceLock};

// TODO:
// Add spans
// Add metadata/callsite recording

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

pub struct Event<'a> {
    pub level: Level,
    pub message: Cow<'a, str>,
}

pub trait Subscriber {
    fn enabled(&self, level: Level) -> bool {
        true
    }

    fn receive_event(&self, event: Event) {}
}

pub struct GlobalSubscriber(pub Box<dyn Subscriber + Send + Sync>);

pub static GLOBAL_SUBSCRIBER: OnceLock<GlobalSubscriber> = OnceLock::new();

pub fn set_subscriber<S>(subscriber: S) -> Result<(), ()>
where
    S: Subscriber + Send + Sync + 'static,
{
    GLOBAL_SUBSCRIBER
        .set(GlobalSubscriber(Box::new(subscriber)))
        .map_err(|_| ())
}

pub fn dispatch_event(event: Event) {
    let Some(subscriber) = GLOBAL_SUBSCRIBER.get() else {
        return;
    };

    subscriber.0.receive_event(event);
}

#[macro_export]
macro_rules! event {
    ($level:expr, $($arg:tt)*) => {{
        if let Some(subscriber) = $crate::trace::GLOBAL_SUBSCRIBER.get() {
            if subscriber.0.enabled($level) {
                subscriber.0.receive_event($crate::trace::Event {
                    level: $level,
                    message: std::borrow::Cow::Owned(format!($($arg)*)),
                })
            }
        }
    }};
}
pub(crate) use event;

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::event!($crate::trace::Level::Debug, $($arg)*)
    };
}
pub(crate) use debug;

#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {{
        $crate::event!($crate::trace::Level::Fatal, $($arg)*);
        std::process::exit(1);
    }};
}
pub(crate) use fatal;
