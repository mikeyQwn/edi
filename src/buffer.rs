use crate::{
    escaping::ANSIColor,
    window::{Cell, Window},
};

pub struct Buffer {
    pub inner: String,
    pub width: usize,
    pub height: usize,
}

impl Buffer {
    #[must_use]
    pub const fn new(inner: String, width: usize, height: usize) -> Self {
        Self {
            inner,
            width,
            height,
        }
    }

    pub fn flush(&self, window: &mut Window, should_wrap: bool) {
        let mut line = 0;
        let mut idx = 0;
        let mut chars = self.inner.chars();
        while let Some(c) = chars.next() {
            if line > self.height {
                break;
            }
            window.put_cell(idx, line, Cell::new(c, ANSIColor::Cyan));
            idx += 1;
            if idx > self.width {
                match should_wrap {
                    true => {
                        for v in chars.by_ref() {
                            if v != '\n' {
                                continue;
                            }
                            line += 1;
                            idx = 0;
                            break;
                        }
                    }
                    false => {
                        line += 1;
                        idx = 0;
                    }
                }
            }
            if c == '\n' {
                line += 1;
                idx = 0;
            }
        }
    }
}
