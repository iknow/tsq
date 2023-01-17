use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

use clap::Parser as ClapParser;
use glob::glob;

use tree_sitter::{Language, Parser, Query, QueryCursor, Tree};

#[derive(clap::Parser)]
struct Cli {
    globs: Vec<String>,
}

extern "C" {
    fn tree_sitter_typescript() -> Language;
    fn tree_sitter_tsx() -> Language;
}

struct LanguageBundle {
    language: Language,
    query: Query,
    query_name_key: usize,
}

impl LanguageBundle {
    fn parse(&self, text: impl AsRef<[u8]>) -> Option<Tree> {
        let mut parser = Parser::new();
        parser.set_language(self.language).unwrap();
        parser.parse(text, None)
    }
}

const QUERY: &str = include_str!("locales.scm");

fn init_languages() -> HashMap<String, Rc<LanguageBundle>> {
    let mut result = HashMap::new();

    let mut add_language = |extensions: &[&str], language: Language| {
        let query = Query::new(language, &QUERY).expect("Construct query");

        let query_name_key: usize = query
            .capture_index_for_name("key")
            .unwrap()
            .try_into()
            .unwrap();

        let bundle = Rc::new(LanguageBundle {
            language,
            query,
            query_name_key,
        });

        for ext in extensions {
            result.insert(ext.to_string(), bundle.clone());
        }
    };

    add_language(&["ts", "js"], unsafe { tree_sitter_typescript() });
    add_language(&["tsx", "jsx"], unsafe { tree_sitter_tsx() });

    return result;
}

fn main() {
    let args = Cli::parse();

    let langs = init_languages();

    for pattern in args.globs {
        for entry in glob(&pattern).expect("Failed to glob inputs") {
            let entry_path = entry.expect("Read glob entry");

            let extension = entry_path.extension().unwrap().to_str().unwrap();
            let lang = langs
                .get(extension)
                .expect(format!("Parser for extension {:?}", extension).as_str());

            let contents = fs::read_to_string(entry_path).expect("Read source file");
            let tree = lang.parse(&contents).expect("Parse source");

            let mut cursor = QueryCursor::new();
            let matches = cursor.matches(&lang.query, tree.root_node(), contents.as_bytes());

            for m in matches {
                let key_node = m.captures[lang.query_name_key].node;
                let key = &contents[key_node.byte_range()];
                println!("{}", key);
            }
        }
    }
}
