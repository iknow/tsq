use std::io::{self, Write};
use std::path::Path;

use tree_sitter::{Query, QueryMatches, TextProvider};

pub mod snippet;
pub mod terse;
pub mod verbose;

pub trait Formatter {
    fn emit_matches<'a, 'tree, T>(
        &self,
        writer: &mut impl Write,
        query: &Query,
        contents: &str,
        file_path: &impl AsRef<Path>,
        matches: QueryMatches<'a, 'tree, T>,
    ) -> io::Result<()>
    where
        T: TextProvider<'a> + 'a,
        'tree: 'a;
}
