pub(crate) fn cut_to_chars(s: &str, n: usize) -> &str {
    let idx = s
        .char_indices()
        .map(|(idx, _)| idx)
        .nth(n)
        .unwrap_or(s.len());
    &s[..idx]
}

#[cfg(test)]
mod test {
    #[test]
    fn cut_to_chars() {
        let cases = ["helo", "◂◆⬔♙❑", "🍋🍌", "🍌abc⬔"];

        for s in cases {
            let total_chars = s.chars().count();
            for n in 0..total_chars {
                let expected: String = s.chars().take(n).collect();
                let got = super::cut_to_chars(s, n);
                assert_eq!(expected, got)
            }
        }
    }
}
