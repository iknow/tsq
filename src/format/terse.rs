use crate::format::Formatter;
use std::path::Path;
use tree_sitter::{Query, QueryMatches, TextProvider};

pub struct Terse {}

impl Formatter for Terse {
    fn emit_matches<'a, 'tree, T>(
        self: &Self,
        query: &Query,
        contents: &str,
        _file_path: &Path,
        matches: QueryMatches<'a, 'tree, T>,
    ) where
        T: TextProvider<'a> + 'a,
    {
        let names = query.capture_names();

        for m in matches {
            let mut data = json::JsonValue::new_object();

            for qc in m.captures {
                let i: usize = qc.index.try_into().unwrap();
                let name = &names[i];
                let match_contents = &contents[qc.node.byte_range()];
                data[name] = match_contents.into();
            }

            println!("{}", data.dump());
        }
    }
}
