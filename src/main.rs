//! Edi the text editor

use libc as _;
#[cfg(test)]
use rand as _;
use termios as _;
use thiserror as _;
use timeout_readwrite as _;

mod app;
mod cli;

fn main() {
    #[cfg(debug_assertions)]
    edi::log::set_debug(true);
    edi::log::set_debug(false);

    let args = cli::EdiCli::parse(std::env::args());
    let err = app::run(args);

    if let Err(e) = err {
        edi::debug!("fatal error: {:?}", e);
    }
}
