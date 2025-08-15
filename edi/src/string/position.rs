#[derive(Debug, Clone, Copy)]
pub enum LinePosition {
    Start,
    CharacterStart,
    CurrentWordEnd,
    CurrentWordStart,
    End,
}

#[derive(Debug, Clone, Copy)]
pub enum GlobalPosition {
    Start,
    End,
}
