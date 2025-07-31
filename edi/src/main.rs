//! Edi the text editor

use std::process::ExitCode;

use edi_lib::trace_subscriber::FileLogSubscriber;
use error::{AppError, AppErrorKind, Result};

use edi_rope as _;
#[cfg(test)]
use rand as _;

mod app;
mod cli;
mod error;

#[cfg(not(debug_assertions))]
fn is_debug() -> bool {
    false
}

#[cfg(debug_assertions)]
fn is_debug() -> bool {
    true
}

const DEBUG_FILE: &str = "log";

fn setup_logging() -> Result<()> {
    let sub = FileLogSubscriber::new(DEBUG_FILE).map_err(|err| {
        AppError::new(
            format!("unable to initialize logging, file `{DEBUG_FILE}` could not be created"),
            AppErrorKind::Io,
        )
        .with_cause(err)
        .with_hint(format!(
            "try adjusting the permissions for file {DEBUG_FILE}"
        ))
    })?;

    edi_lib::trace::set_subscriber(sub).map_err(|_| {
        AppError::new(
            "unable to initialize logging, set_subscriber failed",
            AppErrorKind::Unexpected,
        )
    })?;

    Ok(())
}

fn run() -> Result<()> {
    if is_debug() {
        setup_logging()?;
    }

    let args = cli::EdiCli::parse(std::env::args());
    app::run(args)
        .map_err(|err| AppError::new(format!("fatal error: {err:?}"), AppErrorKind::Unexpected))?;

    Ok(())
}

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
