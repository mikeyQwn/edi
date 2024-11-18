pub struct Buffer {
    pub inner: String,
}

impl Buffer {
    #[must_use]
    pub const fn new(inner: String) -> Self {
        Self { inner }
    }
}
