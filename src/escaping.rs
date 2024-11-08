use std::borrow::Cow;

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
    fn value(self) -> &'static str {
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

pub enum ANSIEscape<'a> {
    ClearScreen,
    MoveTo(usize, usize),
    Write(Cow<'a, str>),
    SetColor(ANSIColor),
}

impl<'a> ANSIEscape<'a> {
    fn value(self) -> Cow<'a, str> {
        match self {
            Self::ClearScreen => Cow::Borrowed("\x1b[2J"),
            Self::MoveTo(x, y) => Cow::Owned(format!("\x1b[{};{}H", y + 1, x + 1)),
            Self::Write(text) => text,
            Self::SetColor(color) => Cow::Borrowed(color.value()),
        }
    }
}

// ANSI escape codes
pub struct EscapeBuilder<'a> {
    inner: Vec<ANSIEscape<'a>>,
}

impl<'a> EscapeBuilder<'a> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn clear_screen(mut self) -> Self {
        self.inner.push(ANSIEscape::ClearScreen);
        self
    }

    pub fn move_to(mut self, x: usize, y: usize) -> Self {
        self.inner.push(ANSIEscape::MoveTo(x, y));
        self
    }

    pub fn set_color(mut self, color: ANSIColor) -> Self {
        self.inner.push(ANSIEscape::SetColor(color));
        self
    }

    pub fn write(mut self, text: Cow<'a, str>) -> Self {
        self.inner.push(ANSIEscape::Write(text));
        self
    }

    pub fn build(self) -> String {
        self.inner
            .into_iter()
            .fold(String::new(), |mut acc, escape| {
                let escape_string = escape.value();
                acc.push_str(&escape_string);
                acc
            })
    }
}
