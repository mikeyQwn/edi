use std::borrow::Cow;

pub enum ANSIEscape<'a> {
    ClearScreen,
    MoveTo(usize, usize),
    Write(Cow<'a, str>),
}

impl<'a> ANSIEscape<'a> {
    fn value(self) -> Cow<'a, str> {
        match self {
            Self::ClearScreen => Cow::Borrowed("\x1b[2J"),
            Self::MoveTo(x, y) => Cow::Owned(format!("\x1b[{};{}H", y + 1, x + 1)),
            Self::Write(text) => text,
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
