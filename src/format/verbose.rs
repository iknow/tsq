use crate::format::Formatter;
use json::object;

use std::path::Path;

use tree_sitter::{Query, QueryMatches};

pub struct Verbose {}

impl Formatter for Verbose {
    fn emit_matches<'a>(
        &self,
        query: &Query,
        contents: &str,
        file_path: &Path,
        matches: QueryMatches<'a, 'a, &'a [u8]>,
    ) {
        let names = query.capture_names();

        let mut matches_json = Vec::new();

        for m in matches {
            let mut captures = json::JsonValue::new_object();

            for qc in m.captures {
                let i: usize = qc.index.try_into().unwrap();
                let name = &names[i];
                let match_contents = &contents[qc.node.byte_range()];
                captures[name] = object! {
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

            matches_json.push(captures);
        }

        let match_obj = object! {
          file: file_path.to_str(),
          matches: matches_json,
        };

        println!("{}", match_obj.dump());
    }
}
