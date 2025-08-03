//! Edi the text editor

use std::process::ExitCode;

use edi_lib::trace_subscriber::FileLogSubscriber;
use error::{AppError, Result};

use edi_rope as _;
#[cfg(test)]
use rand as _;

mod app;
mod cli;
mod error;
mod event;

const DEBUG_FILE: &str = "log";

/// # Errors
///
/// Returns `AppError` if debug file is not available or subscriber is already set
pub fn setup_logging() -> Result<()> {
    let sub = FileLogSubscriber::new(DEBUG_FILE).map_err(|err| {
        AppError::io(format!(
            "unable to initialize logging, file `{DEBUG_FILE}` could not be created"
        ))
        .with_cause(err)
        .with_hint(format!(
            "try adjusting the permissions for file {DEBUG_FILE}"
        ))
    })?;

    if !edi_lib::trace::set_subscriber(sub) {
        return Err(AppError::unexpected(
            "unable to initialize logging, set_subscriber failed",
        ));
    }

    Ok(())
}

fn run() -> Result<()> {
    #[cfg(debug_assertions)]
    setup_logging()?;

    let args = cli::EdiCli::parse(std::env::args())?;
    app::run(args).map_err(|err| AppError::unexpected(format!("fatal error: {err:?}")))?;

    Ok(())
}

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
