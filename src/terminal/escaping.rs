//! Terminal ANSI escape handling

use std::borrow::Cow;

use crate::vec2::Vec2;

/// An ANSI color representation
/// Does not support true color
#[allow(unused)]
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ANSIColor {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl ANSIColor {
    const fn value(self) -> &'static str {
        match self {
            Self::Reset => "\x1b[0m",
            Self::Black => "\x1b[30m",
            Self::Red => "\x1b[31m",
            Self::Green => "\x1b[32m",
            Self::Yellow => "\x1b[33m",
            Self::Blue => "\x1b[34m",
            Self::Magenta => "\x1b[35m",
            Self::Cyan => "\x1b[36m",
            Self::White => "\x1b[37m",
        }
    }
}

/// An ANSI escape code
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ANSIEscape<'a> {
    /// Fully clears the terminal screen
    ClearScreen,
    /// Moves the caret to the given position
    MoveTo(Vec2<usize>),
    /// Writes a string at a given position
    Write(Cow<'a, str>),
    /// Sets the foreground color to the ANSI color
    SetColor(ANSIColor),
    /// Enters the alternate screen state
    EnterAlternateScreen,
    /// Exits the alternate screen state
    ExitAlternateScreen,
}

impl<'a> ANSIEscape<'a> {
    #[must_use]
    pub fn to_str(self) -> Cow<'a, str> {
        match self {
            Self::ClearScreen => Cow::Borrowed("\x1b[2J"),
            Self::MoveTo(pos) => Cow::Owned(format!("\x1b[{};{}H", pos.y + 1, pos.x + 1)),
            Self::Write(text) => text,
            Self::SetColor(color) => Cow::Borrowed(color.value()),
            Self::EnterAlternateScreen => Cow::Borrowed("\x1b[?1049h"),
            Self::ExitAlternateScreen => Cow::Borrowed("\x1b[?1049l"),
        }
    }
}

/// ANSI escape codes builder
/// It is intended to be used to concatenate multiple escape codes and turn them into a string
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct EscapeBuilder<'a> {
    inner: Vec<ANSIEscape<'a>>,
}

impl<'a> EscapeBuilder<'a> {
    /// Constructs a default version of `EscapeBuilder`
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Fully clears the terminal screen
    #[must_use]
    pub fn clear_screen(mut self) -> Self {
        self.inner.push(ANSIEscape::ClearScreen);
        self
    }

    /// Moves the caret to the given position
    #[must_use]
    pub fn move_to(mut self, pos: Vec2<usize>) -> Self {
        self.inner.push(ANSIEscape::MoveTo(pos));
        self
    }

    /// Writes a string at a given position
    #[must_use]
    pub fn write(mut self, text: Cow<'a, str>) -> Self {
        self.inner.push(ANSIEscape::Write(text));
        self
    }

    /// Sets the foreground color to the ANSI color
    #[must_use]
    pub fn set_color(mut self, color: ANSIColor) -> Self {
        self.inner.push(ANSIEscape::SetColor(color));
        self
    }

    /// Concatenates the escape codes from `other` to `self`
    #[must_use]
    pub fn concat<'b>(mut self, other: EscapeBuilder<'b>) -> Self
    where
        'b: 'a,
    {
        self.inner.extend(other.inner);
        self
    }

    /// Constructs a string out of accumulated esape codes
    #[must_use]
    pub fn build(self) -> String {
        self.inner
            .into_iter()
            .fold(String::new(), |mut acc, escape| {
                let escape_string = escape.to_str();
                acc.push_str(&escape_string);
                acc
            })
    }
}
