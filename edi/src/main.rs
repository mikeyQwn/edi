//! Edi the text editor

use edi_rope as _;
#[cfg(test)]
use rand as _;

mod app;
mod cli;

fn main() {
    #[cfg(debug_assertions)]
    edi::log::set_debug(true);
    edi::log::set_debug(false);

    if edi::log::init().is_err() {
        eprintln!("unable to initialize logging");
        return;
    };

    let args = cli::EdiCli::parse(std::env::args());
    let err = app::run(args);

    if let Err(e) = err {
        edi::debug!("fatal error: {:?}", e);
    }
}
