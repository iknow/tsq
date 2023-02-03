use crate::format::Formatter;
use json::object;
use std::path::Path;
use tree_sitter::{Query, QueryMatches, TextProvider};

pub struct Verbose {}

impl Formatter for Verbose {
    fn emit_matches<'a, 'tree, T>(
        self: &Self,
        query: &Query,
        contents: &str,
        file_path: &Path,
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
                data[name] = object! {
                  node: {
                    kind: qc.node.kind(),
                    start_byte: qc.node.start_byte(),
                    end_byte: qc.node.end_byte(),
                    start_position: {
                      row: qc.node.start_position().row,
                      column: qc.node.start_position().column,
                    },
                    end_position: {
                      row: qc.node.end_position().row,
                      column: qc.node.end_position().column,
                    }
                  },
                  content: Into::<String>::into(match_contents),
                }
            }

            let match_obj = object! {
              file: file_path.to_str(),
              matches: data,
            };

            println!("{}", match_obj.dump());
        }
    }
}
