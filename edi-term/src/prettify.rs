use crate::escaping::{ANSIColor, EscapeBuilder};

#[must_use]
pub fn bold(s: &str) -> String {
    EscapeBuilder::new().bold().write_str(s).end_bold().build()
}

#[must_use]
pub fn italic(s: &str) -> String {
    EscapeBuilder::new()
        .italic()
        .write_str(s)
        .end_italic()
        .build()
}

#[must_use]
pub fn underline(s: &str) -> String {
    EscapeBuilder::new()
        .underline()
        .write_str(s)
        .end_underline()
        .build()
}

#[must_use]
pub fn red(s: &str) -> String {
    EscapeBuilder::new()
        .set_color(ANSIColor::Red)
        .write_str(s)
        .end_color()
        .build()
}
