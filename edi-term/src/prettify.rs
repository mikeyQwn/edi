use crate::escaping::{ANSIColor, EscapeBuilder};

pub fn bold(s: &str) -> String {
    EscapeBuilder::new().bold().write_str(s).end_bold().build()
}

pub fn italic(s: &str) -> String {
    EscapeBuilder::new()
        .italic()
        .write_str(s)
        .end_italic()
        .build()
}

pub fn underline(s: &str) -> String {
    EscapeBuilder::new()
        .underline()
        .write_str(s)
        .end_underline()
        .build()
}

pub fn red(s: &str) -> String {
    EscapeBuilder::new()
        .set_color(ANSIColor::Red)
        .write_str(s)
        .end_color()
        .build()
}
