//! Edi's own library for string and terminal manipulation

// TODO: Turn back on when finished buffer refactor
//#![deny(missing_docs)]

pub mod buffer;
pub mod draw;
pub mod rect;
pub mod string;

use anyhow as _;
use smallvec as _;
