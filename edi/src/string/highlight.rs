//! Highlighting utilities

use edi_rope::Rope;

use crate::fs::filetype::{self, Filetype};

/// A type of the highlight
///
/// Used to determine which color to apply
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// Function name in definition / call
    Function,
    /// A keyword, e.g "let, const, var"
    Keyword,
    /// Identifier, e.g Variable name
    Identifier,
    /// A type definition
    Type,
    /// A comment
    Comment,
}

/// Represents a chunk of characters that should be highlighed grouped by highlihght type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Highlight {
    /// Character number of the start of highlighed word or symbol
    pub start: usize,
    /// Length of the highlighted word or symbol
    pub len: usize,
    /// Type of the highlight, for more information, see `Type`
    pub ty: Type,
}

impl PartialOrd for Highlight {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.start.cmp(&other.start))
    }
}

impl Ord for Highlight {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

fn get_line_highlights(line: &str, keywords: &[(&str, Type)]) -> Vec<Highlight> {
    let mut line_highlights = Vec::new();
    for &(word, ty) in keywords {
        line_highlights.extend(
            line.match_indices(word)
                .map(|(idx, _)| (idx, idx + word.len()))
                .filter(|&(start, end)| {
                    let starts_with_not_alphanum = start == 0
                        || line
                            .chars()
                            .nth(start - 1)
                            .filter(|&c| c.is_alphanumeric() || c == '_')
                            .is_none();

                    let ends_with_not_alphanum = line
                        .chars()
                        .nth(end)
                        .filter(|&c| c.is_alphanumeric() || c == '_')
                        .is_none();

                    starts_with_not_alphanum && ends_with_not_alphanum
                })
                .map(|(start, _)| Highlight {
                    start,
                    len: word.len(),
                    ty,
                }),
        );
    }

    line_highlights.sort();
    line_highlights
}

/// Get all highlights of `contents` based on the `filetype`. Highlights are sorted by default
#[must_use]
pub fn get_highlights(content: &Rope, filetype: &Filetype) -> Vec<Highlight> {
    let kw = filetype_to_keywords(filetype);
    content
        .lines()
        .flat_map(|line| {
            let mut highlights = get_line_highlights(&line.contents, kw);
            highlights
                .iter_mut()
                .for_each(|v| v.start += line.character_offset);
            highlights
        })
        .collect()
}

fn filetype_to_keywords<'b, 'c>(ft: &Filetype) -> &'b [(&'c str, Type)] {
    if ft.eq(&filetype::C) {
        return &C_KEYWORDS;
    }

    if ft.eq(&filetype::RUST) {
        return &RUST_KEYWORDS;
    }

    &[]
}

const C_KEYWORDS: [(&str, Type); 32] = [
    ("auto", Type::Keyword),
    ("break", Type::Keyword),
    ("case", Type::Keyword),
    ("char", Type::Keyword),
    ("const", Type::Keyword),
    ("continue", Type::Keyword),
    ("default", Type::Keyword),
    ("do", Type::Keyword),
    ("double", Type::Keyword),
    ("else", Type::Keyword),
    ("enum", Type::Keyword),
    ("extern", Type::Keyword),
    ("float", Type::Keyword),
    ("for", Type::Keyword),
    ("if", Type::Keyword),
    ("int", Type::Keyword),
    ("long", Type::Keyword),
    ("register", Type::Keyword),
    ("return", Type::Keyword),
    ("short", Type::Keyword),
    ("signed", Type::Keyword),
    ("sizeof", Type::Keyword),
    ("static", Type::Keyword),
    ("struct", Type::Keyword),
    ("switch", Type::Keyword),
    ("typedef", Type::Keyword),
    ("union", Type::Keyword),
    ("unsigned", Type::Keyword),
    ("void", Type::Keyword),
    ("goto", Type::Keyword),
    ("volatile", Type::Keyword),
    ("while", Type::Keyword),
];

const RUST_KEYWORDS: [(&str, Type); 53] = [
    ("as", Type::Keyword),
    ("break", Type::Keyword),
    ("const", Type::Keyword),
    ("continue", Type::Keyword),
    ("crate", Type::Keyword),
    ("else", Type::Keyword),
    ("enum", Type::Keyword),
    ("extern", Type::Keyword),
    ("false", Type::Keyword),
    ("fn", Type::Keyword),
    ("for", Type::Keyword),
    ("if", Type::Keyword),
    ("impl", Type::Keyword),
    ("in", Type::Keyword),
    ("let", Type::Keyword),
    ("loop", Type::Keyword),
    ("match", Type::Keyword),
    ("mod", Type::Keyword),
    ("move", Type::Keyword),
    ("mut", Type::Keyword),
    ("pub", Type::Keyword),
    ("ref", Type::Keyword),
    ("return", Type::Keyword),
    ("self", Type::Keyword),
    ("Self", Type::Keyword),
    ("static", Type::Keyword),
    ("struct", Type::Keyword),
    ("super", Type::Keyword),
    ("trait", Type::Keyword),
    ("true", Type::Keyword),
    ("type", Type::Keyword),
    ("unsafe", Type::Keyword),
    ("use", Type::Keyword),
    ("where", Type::Keyword),
    ("while", Type::Keyword),
    ("async", Type::Keyword),
    ("await", Type::Keyword),
    ("dyn", Type::Keyword),
    ("i8", Type::Type),
    ("i16", Type::Type),
    ("i32", Type::Type),
    ("i64", Type::Type),
    ("i128", Type::Type),
    ("u8", Type::Type),
    ("u16", Type::Type),
    ("u32", Type::Type),
    ("u64", Type::Type),
    ("u128", Type::Type),
    ("f32", Type::Type),
    ("f64", Type::Type),
    ("bool", Type::Type),
    ("usize", Type::Type),
    ("isize", Type::Type),
];
