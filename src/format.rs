use std::path::Path;

use tree_sitter::{Query, QueryMatches, TextProvider};

pub mod snippet;
pub mod terse;
pub mod verbose;

pub trait Formatter<'a, 'tree, T>
where
    T: TextProvider<'a> + 'a,
    'tree: 'a,
{
    fn emit_matches(
        &self,
        query: &Query,
        contents: &str,
        file_path: &Path,
        matches: QueryMatches<'a, 'tree, T>,
    );
}
