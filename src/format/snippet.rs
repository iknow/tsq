use crate::format::Formatter;

use std::io::{Result, Write};
use std::{ops::Range, path::Path};

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use tree_sitter::TextProvider;

pub struct SnippetFormatter {}

impl<'a, 'tree, T, Writer> Formatter<'a, 'tree, T, Writer> for SnippetFormatter
where
    Writer: Write,
    T: TextProvider<'a> + 'a,
    'tree: 'a,
{
    fn emit_matches(
        &self,
        writer: &mut Writer,
        query: &tree_sitter::Query,
        contents: &str,
        file_path: &Path,
        matches: tree_sitter::QueryMatches<'a, 'tree, T>,
    ) -> Result<()> {
        let names = query.capture_names();

        let mut slices = Vec::new();

        for m in matches {
            let mut annotations = Vec::new();

            // Trim the source down to the containing line. It should be
            // sufficient to search backwards and forwards to the next line
            // feed character.

            let earliest_byte = m
                .captures
                .iter()
                .map(|x| x.node.start_byte())
                .min()
                .unwrap();

            let latest_byte = m.captures.iter().map(|x| x.node.end_byte()).max().unwrap();

            let source_min_byte = contents[..earliest_byte]
                .rfind('\n')
                .map(|x| x + 1)
                .unwrap_or(0);

            let source_max_byte =
                contents[latest_byte..].find('\n').unwrap_or(contents.len()) + latest_byte;

            let source_range: Range<usize> = source_min_byte..source_max_byte;

            let line_start = m
                .captures
                .iter()
                .map(|x| x.node.start_position().row)
                .min()
                .unwrap();

            for qc in m.captures {
                let i: usize = qc.index.try_into().unwrap();
                annotations.push(SourceAnnotation {
                    annotation_type: AnnotationType::Info,
                    label: &names[i],
                    range: (
                        qc.node.start_byte() - source_min_byte,
                        qc.node.end_byte() - source_min_byte,
                    ),
                })
            }

            slices.push(Slice {
                source: &contents[source_range],
                line_start: line_start + 1,
                origin: file_path.to_str(),
                annotations,
                fold: true,
            })
        }

        if slices.is_empty() {
            return Ok(());
        }

        let snippet = Snippet {
            title: Some(Annotation {
                label: Some("Query matched"),
                id: None,
                annotation_type: AnnotationType::Info,
            }),
            footer: vec![],
            slices,
            opt: FormatOptions {
                color: true,
                ..Default::default()
            },
        };

        writeln!(writer, "{}", DisplayList::from(snippet))?;
        Ok(())
    }
}
