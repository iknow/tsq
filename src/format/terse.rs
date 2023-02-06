use crate::format::Formatter;

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::Path;

use tree_sitter::{Query, QueryMatches, TextProvider};

pub struct Terse {}

impl Formatter for Terse {
    fn emit_matches<'a, 'tree, T>(
        &self,
        writer: &mut impl Write,
        query: &Query,
        contents: &str,
        _file_path: &impl AsRef<Path>,
        matches: QueryMatches<'a, 'tree, T>,
    ) -> io::Result<()>
    where
        T: TextProvider<'a> + 'a,
        'tree: 'a,
    {
        let names = query.capture_names();

        for m in matches {
            let mut data = BTreeMap::<&'a str, &'a str>::new();

            for qc in m.captures {
                let i: usize = qc.index.try_into().unwrap();
                let name = &names[i];
                let match_contents = &contents[qc.node.byte_range()];
                data.insert(name, match_contents);
            }

            serde_json::to_writer(&mut *writer, &data)?;
            writeln!(&mut *writer)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::format::terse::Terse;
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
            &Terse {},
            &PathBuf::from("examples/mnoga/index.spec.ts"),
            &lang,
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string("examples/mnoga/terse.expected").unwrap(),
            String::from_utf8(result).unwrap(),
        );
    }
}
