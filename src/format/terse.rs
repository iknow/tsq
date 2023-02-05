use crate::format::Formatter;

use std::collections::HashMap;
use std::path::Path;

use tree_sitter::{Query, QueryMatches, TextProvider};

pub struct Terse {}

impl<'a, 'tree, T> Formatter<'a, 'tree, T> for Terse
where
    T: TextProvider<'a> + 'a,
    'tree: 'a,
{
    fn emit_matches(
        &self,
        query: &Query,
        contents: &str,
        _file_path: &Path,
        matches: QueryMatches<'a, 'tree, T>,
    ) {
        let names = query.capture_names();

        for m in matches {
            let mut data = HashMap::<String, String>::new();

            for qc in m.captures {
                let i: usize = qc.index.try_into().unwrap();
                let name = &names[i];
                let match_contents = &contents[qc.node.byte_range()];
                data.insert(name.into(), match_contents.into());
            }

            println!("{}", serde_json::to_string(&data).unwrap());
        }
    }
}
