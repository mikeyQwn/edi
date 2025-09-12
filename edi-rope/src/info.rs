#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct TextInfo {
    pub(crate) bytes: usize,
    pub(crate) chars: usize,
    pub(crate) newlines: usize,
}

impl TextInfo {
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn from_str(text: &str) -> Self {
        let bytes = text.len();
        let (chars, newlines) = text.chars().fold((0, 0), |(chars, newlines), c| {
            (chars + 1, newlines + (c == '\n') as usize)
        });

        Self {
            bytes,
            chars,
            newlines,
        }
    }
}

impl From<&str> for TextInfo {
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::info::TextInfo;

    #[test]
    fn creation() {
        let strings = ["hello\nмир", "alskdjf", "flksaj\n\naskjdf", "\nlaksjdf"];
        for string in strings {
            let info = TextInfo::from(string);
            assert_eq!(string.chars().filter(|&c| c == '\n').count(), info.newlines);
            assert_eq!(string.chars().count(), info.chars);
            assert_eq!(string.len(), info.bytes);
        }
    }
}
