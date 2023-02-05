use crate::format::Formatter;

use std::collections::BTreeMap;
use std::io::{self, Write};
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
    matches: Vec<BTreeMap<String, Capture>>,
}

impl Formatter for Verbose {
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
        'tree: 'a,
    {
        let names = query.capture_names();

        let mut matches_json = Vec::new();

        for m in matches {
            let mut captures = BTreeMap::new();

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

        let file_path: &Path = file_path.as_ref();
        let match_obj = Matches {
            file: file_path.to_str().map(|s| s.into()),
            matches: matches_json,
        };

        serde_json::to_writer(&mut *writer, &match_obj)?;
        writeln!(&mut *writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::format::verbose::Verbose;
    use crate::{get_language, process_file};
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_mnoga() {
        let lang = get_language("typescript", &PathBuf::from("examples/mnoga/locales.scm"));

        let mut result = Vec::new();

        process_file(
            &mut result,
            &Verbose {},
            &PathBuf::from("examples/mnoga/index.spec.ts"),
            &lang,
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string("examples/mnoga/verbose.expected").unwrap(),
            String::from_utf8(result).unwrap(),
        );
    }
}
