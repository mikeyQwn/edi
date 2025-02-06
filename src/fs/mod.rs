//! Filesystem utilities
use std::rc::Rc;

pub(super) const UNKNOWN_FILETYPE: &'static str = "unknown";
pub(super) const C_FILETYPE: &'static str = "c";
pub(super) const CPP_FILETYPE: &'static str = "cpp";
pub(super) const GO_FILETYPE: &'static str = "go";
pub(super) const RUST_FILETYPE: &'static str = "rust";
pub(super) const MARKDOWN_FILETYPE: &'static str = "markdown";

/// A struct representing a filetype
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Filetype(pub(super) Rc<str>);

impl Clone for Filetype {
    fn clone(&self) -> Self {
        Filetype(Rc::clone(&self.0))
    }
}

impl Default for Filetype {
    fn default() -> Self {
        Filetype(Rc::from("unknown"))
    }
}

impl Filetype {
    /// Extracts the filetype from an extension (with leading `.` removed), regardless if it's
    /// known or not
    pub fn from_ext(ext: &str) -> Self {
        Self::from_known_ext(ext).unwrap_or(Self(Rc::from(ext)))
    }

    /// Tries to map an extenstion to it's filetype, if known
    pub fn from_known_ext(ext: &str) -> Option<Self> {
        let inner = match ext {
            "c" | "h" => C_FILETYPE,
            "cpp" | "hpp" => CPP_FILETYPE,
            "go" => GO_FILETYPE,
            "rs" => RUST_FILETYPE,
            "md" => MARKDOWN_FILETYPE,
            _ => {
                return None;
            }
        };

        Some(Self(Rc::from(inner)))
    }
}
