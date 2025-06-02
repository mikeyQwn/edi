//! Edi's own library for string and terminal manipulation

// TODO: Turn back on when finished buffer refactor
//#![deny(missing_docs)]

pub mod buffer;
pub mod draw;
pub mod fs;
pub mod log;
pub mod rect;
pub mod string;
pub mod terminal;
pub mod trace;
pub mod vec2;

use anyhow as _;
