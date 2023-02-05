use crate::format::Formatter;

use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;
use serde_json;
use tree_sitter::{Query, QueryMatches, TextProvider};

pub struct Verbose {}

#[derive(Serialize)]
struct Point {
    row: usize,
    column: usize,
}

#[derive(Serialize)]
struct Node {
    kind: String,
    start_byte: usize,
    end_byte: usize,
    start_position: Point,
    end_position: Point,
}

#[derive(Serialize)]
struct Capture {
    content: String,
    node: Node,
}

#[derive(Serialize)]
struct Matches {
    file: Option<String>,
    matches: Vec<HashMap<String, Capture>>,
}

impl<'a, 'tree, T> Formatter<'a, 'tree, T> for Verbose
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
    ) {
        let names = query.capture_names();

        let mut matches_json = Vec::new();

        for m in matches {
            let mut captures = HashMap::new();

            for qc in m.captures {
                let i: usize = qc.index.try_into().unwrap();
                let name = &names[i];
                let match_contents = &contents[qc.node.byte_range()];

                captures.insert(
                    name.into(),
                    Capture {
                        node: Node {
                            kind: qc.node.kind().into(),
                            start_byte: qc.node.start_byte(),
                            end_byte: qc.node.end_byte(),
                            start_position: Point {
                                row: qc.node.start_position().row,
                                column: qc.node.start_position().column,
                            },
                            end_position: Point {
                                row: qc.node.end_position().row,
                                column: qc.node.end_position().column,
                            },
                        },
                        content: match_contents.into(),
                    },
                );
            }

            matches_json.push(captures);
        }

        let match_obj = Matches {
            file: file_path.to_str().map(|s| s.into()),
            matches: matches_json,
        };

        println!("{}", serde_json::to_string(&match_obj).unwrap());
    }
}
