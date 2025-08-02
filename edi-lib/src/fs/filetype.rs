//! Common filetype operations and constants

use std::{
    ffi::OsStr,
    sync::{Arc, LazyLock},
};

pub static UNKNOWN: LazyLock<Filetype> = LazyLock::new(|| Filetype(Arc::from("unknown")));
pub static C: LazyLock<Filetype> = LazyLock::new(|| Filetype(Arc::from("c")));
pub static CPP: LazyLock<Filetype> = LazyLock::new(|| Filetype(Arc::from("cpp")));
pub static GO: LazyLock<Filetype> = LazyLock::new(|| Filetype(Arc::from("go")));
pub static RUST: LazyLock<Filetype> = LazyLock::new(|| Filetype(Arc::from("rust")));
pub static MARKDOWN: LazyLock<Filetype> = LazyLock::new(|| Filetype(Arc::from("markdown")));

/// A struct representing a filetype
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Filetype(pub(super) Arc<str>);

impl Clone for Filetype {
    fn clone(&self) -> Self {
        Filetype(Arc::clone(&self.0))
    }
}

impl Default for Filetype {
    fn default() -> Self {
        Filetype::clone(&UNKNOWN)
    }
}

impl Filetype {
    /// Extracts the filetype from an extension (with leading `.` removed), regardless if it's
    /// known or not
    #[must_use]
    pub fn from_ext(ext: &str) -> Self {
        Self::from_known_ext(ext).unwrap_or(Self(Arc::from(ext)))
    }

    /// Tries to map an extenstion to it's filetype, if known
    #[must_use]
    pub fn from_known_ext(ext: &str) -> Option<Self> {
        let inner = match ext {
            "c" | "h" => &C,
            "cpp" | "hpp" => &CPP,
            "go" => &GO,
            "rs" => &RUST,
            "md" => &MARKDOWN,
            _ => {
                return None;
            }
        };

        Some(Self::clone(inner))
    }
}

impl<P> From<P> for Filetype
where
    P: AsRef<std::path::Path>,
{
    fn from(value: P) -> Self {
        let p = value.as_ref();
        p.extension()
            .and_then(OsStr::to_str)
            .map_or_else(|| UNKNOWN.clone(), Self::from_ext)
    }
}
