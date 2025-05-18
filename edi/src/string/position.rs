#[derive(Debug, Clone, Copy)]
pub enum LinePosition {
    Start,
    CharacterStart,
    CurrentWordEnd,
    End,
}

#[derive(Debug, Clone, Copy)]
pub enum GlobalPosition {
    Start,
    End,
}
