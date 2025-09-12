use crate::info::TextInfo;

const LEAF_SIZE: usize = 128;

pub struct Leaf {
    /// A part of the string that the rope represents
    /// INVARIANT: &value[..info.bytes] is always a valid utf-8 string
    value: [u8; LEAF_SIZE],
    /// Information about the text contained in `value`
    info: TextInfo,
}

impl Leaf {
    pub fn new(s: &str) -> Self {
        assert!(s.len() <= LEAF_SIZE);
        let mut value = [0; LEAF_SIZE];
        value.copy_from_slice(s.as_bytes());
        let info = TextInfo::from(s);
        Self { value, info }
    }

    pub fn empty() -> Self {
        let value = [0; LEAF_SIZE];
        let info = TextInfo::default();
        Self { value, info }
    }

    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.value[..self.info.bytes])
            .expect("the invariant of leaf being always valid string must be upheld")
    }

    pub const fn weight(&self) -> usize {
        self.info.chars
    }

    pub const fn newlines(&self) -> usize {
        self.info.newlines
    }

    pub const fn info(&self) -> &TextInfo {
        &self.info
    }
}
