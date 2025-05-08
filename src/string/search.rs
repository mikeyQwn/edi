/// Returns the character offset of the first non-whitespace character in a line
#[must_use]
pub fn character_start(s: &str) -> usize {
    s.chars()
        .enumerate()
        .find_map(|(i, c)| (!c.is_whitespace()).then_some(i))
        .unwrap_or(0)
}
