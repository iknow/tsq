use std::path::Path;

use tree_sitter::{Query, QueryMatches};

mod terse;
mod verbose;

pub trait Formatter {
    fn emit_matches<'a>(
        &self,
        query: &Query,
        contents: &str,
        file_path: &Path,
        matches: QueryMatches<'a, 'a, &'a [u8]>,
    );
}

pub fn terse() -> impl Formatter {
    terse::Terse {}
}

pub fn verbose() -> impl Formatter {
    verbose::Verbose {}
}
