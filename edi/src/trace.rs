use std::{
    borrow::Cow,
    cell::RefCell,
    fmt::{Debug, Display},
    sync::OnceLock,
};

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

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub name: &'static str,
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

thread_local! {
    pub static SPAN_POOL: RefCell<Vec<Span>> = const {RefCell::new(Vec::new())};
}

#[derive(Debug)]
pub struct SpanGuard;

impl Drop for SpanGuard {
    fn drop(&mut self) {
        exit_span();
    }
}

#[must_use]
pub fn enter_span(span: Span) -> SpanGuard {
    SPAN_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        pool.push(span);
    });
    SpanGuard
}

pub fn exit_span() {
    SPAN_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        let _ = pool.pop();
    });
}

#[derive(Debug)]
pub struct Event<'a, 'b> {
    pub level: Level,
    pub spans: &'b [Span],
    pub message: Cow<'a, str>,
}

pub trait Subscriber {
    fn enabled(&self, level: Level) -> bool {
        let _ = level;
        true
    }

    fn receive_event(&self, event: Event);
}

pub struct GlobalSubscriber(pub Box<dyn Subscriber + Send + Sync>);

impl Debug for GlobalSubscriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalSubscriber").finish()
    }
}

pub static GLOBAL_SUBSCRIBER: OnceLock<GlobalSubscriber> = OnceLock::new();

/// # Errors
///
/// Returns an error when a subscriber is already set
///
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
                $crate::trace::SPAN_POOL.with(|pool| {
                let pool = pool.borrow();
                subscriber.0.receive_event($crate::trace::Event {
                    level: $level,
                    spans: &pool,
                    message: std::borrow::Cow::Owned(format!($($arg)*)),
                })
            });
            }
        }
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::event!($crate::trace::Level::Debug, $($arg)*)
    };
}

#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {{
        $crate::event!($crate::trace::Level::Fatal, $($arg)*);
        std::process::exit(1);
    }};
}

#[macro_export]
macro_rules! span {
    ($name:expr) => {
        $crate::trace::enter_span($crate::trace::Span { name: $name })
    };
}
