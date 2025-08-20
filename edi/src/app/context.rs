/// Global app context that should be passed to almost every function
#[derive(Debug, Default)]
pub struct Context {
    pub settings: Settings,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Configurable editor behaviour
#[derive(Debug)]
pub struct Settings {
    pub line_numbers: bool,
    pub word_wrap: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            line_numbers: true,
            word_wrap: true,
        }
    }
}
