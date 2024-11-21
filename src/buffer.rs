use crate::{
    escaping::ANSIColor,
    vec2::Vec2,
    window::{Cell, Window},
};

pub struct Buffer {
    pub inner: String,
    pub size: Vec2<usize>,
}

impl Buffer {
    #[must_use]
    pub const fn new(inner: String, size: Vec2<usize>) -> Self {
        Self { inner, size }
    }

    pub fn flush(&self, window: &mut Window, should_wrap: bool) {
        let mut pos = Vec2::new(0, 0);
        let mut chars = self.inner.chars();
        while let Some(c) = chars.next() {
            if pos.y > self.size.y {
                break;
            }
            window.put_cell(Vec2::new(pos.x, pos.y), Cell::new(c, ANSIColor::Cyan));
            pos.x += 1;
            if pos.x > self.size.x {
                match should_wrap {
                    true => {
                        for v in chars.by_ref() {
                            if v != '\n' {
                                continue;
                            }
                            pos.y += 1;
                            pos.x = 0;
                            break;
                        }
                    }
                    false => {
                        pos.y += 1;
                        pos.x = 0;
                    }
                }
            }
            if c == '\n' {
                pos.y += 1;
                pos.x = 0;
            }
        }
    }
}
