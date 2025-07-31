use std::{error::Error, fmt::Display};

use edi_term::escaping::{ANSIColor, EscapeBuilder};

#[derive(Debug, Clone)]
pub struct DisplayOptions {
    show_cause: bool,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self { show_cause: false }
    }
}

#[derive(Debug)]
pub struct AppError {
    pub message: Box<str>,
    pub cause: Option<Box<dyn Error + Send + Sync>>,
    pub kind: AppErrorKind,
    pub hint: Option<Box<str>>,
}

#[derive(Debug, Clone, Copy)]
pub enum AppErrorKind {
    Io,
    TerminalIo,
    Unexpected,
    InvalidArgument,
}

impl AppErrorKind {
    pub fn error_message(self) -> &'static str {
        match self {
            AppErrorKind::Io => "unable to perform i/o operation",
            AppErrorKind::TerminalIo => "unable to perform i/o operation on the terminal",
            AppErrorKind::Unexpected => "unexpected error occurred",
            AppErrorKind::InvalidArgument => "invalid argument supplied",
        }
    }
}

impl AppError {
    #[must_use]
    pub fn new(message: impl Into<Box<str>>, kind: AppErrorKind) -> Self {
        Self {
            message: message.into(),
            kind,
            cause: None,
            hint: None,
        }
    }

    #[must_use]
    pub fn from_kind(kind: AppErrorKind) -> Self {
        Self::new(kind.error_message(), kind)
    }

    #[must_use]
    pub fn with_cause<E>(mut self, cause: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        self.cause = Some(Box::new(cause));
        self
    }

    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<Box<str>>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    #[must_use]
    pub fn build_error(&self, opts: DisplayOptions) -> String {
        let mut s = EscapeBuilder::new()
            .bold()
            .write_str("[edi]: ")
            .set_color(ANSIColor::Red)
            .write_str("error: ")
            .end_color()
            .write_str(&self.message)
            .end_bold();

        if let Some(cause) = &self.cause {
            if opts.show_cause {
                s = s
                    .write_str("\n\tcaused by: ")
                    .write_string(cause.to_string());
            }
        }

        if let Some(hint) = &self.hint {
            s = s
                .write_str("\n\t")
                .underline()
                .write_str("hint")
                .end_underline()
                .write_str(": ")
                .write_str(hint);
        }

        s.build()
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.build_error(DisplayOptions::default()))
    }
}

impl Error for AppError {
    fn cause(&self) -> Option<&dyn Error> {
        self.cause.as_ref().map(|e| &**e as _)
    }
}

pub type Result<T> = core::result::Result<T, AppError>;
