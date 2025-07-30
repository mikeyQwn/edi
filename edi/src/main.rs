//! Edi the text editor

use std::process::ExitCode;

use edi_rope as _;
#[cfg(test)]
use rand as _;

mod app;
mod cli;

#[cfg(not(debug_assertions))]
fn is_debug() -> bool {
    false
}

#[cfg(debug_assertions)]
fn is_debug() -> bool {
    false
}

const DEBUG_FILE: &str = "log";

fn setup_logging() -> Option<ExitCode> {
    let Ok(log_subscriber) = edi_lib::trace_subscriber::FileLogSubscriber::new(DEBUG_FILE) else {
        eprintln!(
            "edi: [e] unable to initialize logging, file {DEBUG_FILE} could not be written to"
        );
        return Some(ExitCode::FAILURE);
    };
    if edi_lib::trace::set_subscriber(log_subscriber).is_err() {
        eprintln!("edi: [e] unable to initialize logging, set_subscriber failed");
        return Some(ExitCode::FAILURE);
    }

    None
}

fn main() -> ExitCode {
    if is_debug() {
        if let Some(exit_code) = setup_logging() {
            return exit_code;
        }
    }

    let args = cli::EdiCli::parse(std::env::args());
    let err = app::run(args);

    if let Err(e) = err {
        edi_lib::debug!("fatal error: {:?}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
