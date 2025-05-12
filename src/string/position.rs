#[derive(Debug, Clone, Copy)]
pub enum LinePosition {
    Start,
    CharacterStart,
    End,
}

#[derive(Debug, Clone, Copy)]
pub enum GlobalPosition {
    Start,
    End,
}
