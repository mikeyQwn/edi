//! Terminal ANSI escape handling

use std::borrow::Cow;

use crate::coord::Coord;

/// An ANSI color representation
/// Does not support true color
#[allow(unused)]
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ANSIColor {
    Reset,
    Default,
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
            Self::Default => "\x1b[39m",
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

    const fn value_bg(self) -> &'static str {
        match self {
            Self::Reset => "\x1b[0m",
            Self::Default => "\x1b[49m",
            Self::Black => "\x1b[40m",
            Self::Red => "\x1b[41m",
            Self::Green => "\x1b[42m",
            Self::Yellow => "\x1b[43m",
            Self::Blue => "\x1b[44m",
            Self::Magenta => "\x1b[45m",
            Self::Cyan => "\x1b[46m",
            Self::White => "\x1b[47m",
        }
    }
}

/// An ANSI escape code
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ANSIEscape<'a> {
    /// Fully clears the terminal screen
    ClearScreen,
    /// Moves the caret to the given position
    MoveTo(Coord),
    /// Writes a string at a given position
    Write(Cow<'a, str>),
    /// Sets the foreground color to the ANSI color
    SetColor(ANSIColor),
    /// Sets the backgrounod color to the ANSI color
    SetBgColor(ANSIColor),
    /// Makes the following text bold
    StartBold,
    /// Makes the following text NOT bold
    EndBold,
    /// Makes the following text italic
    StartItalic,
    /// Makes the following text NOT italic
    EndItalic,
    /// Makes the following text underlined
    StartUnderline,
    /// Makes the following text NOT underlined
    EndUnderline,
    /// Resets the styles for all the following text
    EndAll,
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
            Self::SetBgColor(color) => Cow::Borrowed(color.value_bg()),
            Self::StartBold => Cow::Borrowed("\x1b[1m"),
            Self::EndBold => Cow::Borrowed("\x1b[22m"),
            Self::StartItalic => Cow::Borrowed("\x1b[3m"),
            Self::EndItalic => Cow::Borrowed("\x1b[23m"),
            Self::StartUnderline => Cow::Borrowed("\x1b[4m"),
            Self::EndUnderline => Cow::Borrowed("\x1b[24m"),
            Self::EndAll => Cow::Borrowed("\x1b[0m"),
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
    pub fn move_to(mut self, pos: Coord) -> Self {
        self.inner.push(ANSIEscape::MoveTo(pos));
        self
    }

    /// Writes a string at the end of escape sequence
    #[must_use]
    pub fn write(mut self, text: Cow<'a, str>) -> Self {
        self.inner.push(ANSIEscape::Write(text));
        self
    }

    /// Writes a &str at the end of escape sequence
    #[must_use]
    pub fn write_str(self, text: &'a str) -> Self {
        self.write(Cow::Borrowed(text))
    }

    /// Writes a &str at the end of escape sequence
    #[must_use]
    pub fn write_string(self, text: String) -> Self {
        self.write(Cow::Owned(text))
    }

    /// Sets the foreground color to the ANSI color
    #[must_use]
    pub fn set_color(mut self, color: ANSIColor) -> Self {
        self.inner.push(ANSIEscape::SetColor(color));
        self
    }

    /// Resets the foreground color to the default
    #[must_use]
    pub fn end_color(mut self) -> Self {
        self.inner.push(ANSIEscape::SetColor(ANSIColor::Default));
        self
    }

    /// Sets the background color to the ANSI color
    #[must_use]
    pub fn set_bg_color(mut self, color: ANSIColor) -> Self {
        self.inner.push(ANSIEscape::SetBgColor(color));
        self
    }

    /// Resets the background color to the default
    #[must_use]
    pub fn end_bg_color(mut self) -> Self {
        self.inner.push(ANSIEscape::SetBgColor(ANSIColor::Default));
        self
    }

    /// Makes the following text bold
    #[must_use]
    pub fn bold(mut self) -> Self {
        self.inner.push(ANSIEscape::StartBold);
        self
    }

    /// Makes the following text NOT bold
    #[must_use]
    pub fn end_bold(mut self) -> Self {
        self.inner.push(ANSIEscape::EndBold);
        self
    }

    /// Makes the following text italic
    #[must_use]
    pub fn italic(mut self) -> Self {
        self.inner.push(ANSIEscape::StartItalic);
        self
    }

    /// Makes the following text NOT italic
    #[must_use]
    pub fn end_italic(mut self) -> Self {
        self.inner.push(ANSIEscape::EndItalic);
        self
    }

    /// Makes the following text underlined
    #[must_use]
    pub fn underline(mut self) -> Self {
        self.inner.push(ANSIEscape::StartUnderline);
        self
    }

    /// Makes the following text NOT underlined
    #[must_use]
    pub fn end_underline(mut self) -> Self {
        self.inner.push(ANSIEscape::EndUnderline);
        self
    }

    /// Resets the styles for the following text
    #[must_use]
    pub fn reset(mut self) -> Self {
        self.inner.push(ANSIEscape::EndAll);
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
