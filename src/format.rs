use std::path::Path;
use tree_sitter::{Query, QueryMatches, TextProvider};

mod terse;
mod verbose;

pub trait Formatter {
    fn emit_matches<'a, 'tree, T>(
        self: &Self,
        query: &Query,
        contents: &str,
        file_path: &Path,
        matches: QueryMatches<'a, 'tree, T>,
    ) where
        T: TextProvider<'a> + 'a;
}

pub fn terse() -> impl Formatter {
    return terse::Terse {};
}

pub fn verbose() -> impl Formatter {
    return verbose::Verbose {};
}
