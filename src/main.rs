use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser as ClapParser;
use glob::glob;
use json;
use tree_sitter::{Language, Parser, Query, QueryCursor, Tree};

#[derive(clap::Parser)]
struct Cli {
    #[arg(short, long)]
    query: PathBuf,

    #[arg(short, long)]
    languages: Vec<String>,

    globs: Vec<String>,
}

struct LanguageBundle {
    language: Language,
    query: Query,
}

impl LanguageBundle {
    fn parse(&self, text: impl AsRef<[u8]>) -> Option<Tree> {
        let mut parser = Parser::new();
        parser.set_language(self.language).unwrap();
        parser.parse(text, None)
    }
}

struct LanguageLoader {
    dir: PathBuf,
    cache: HashMap<String, Language>,
}

impl LanguageLoader {
    fn new(dir: PathBuf) -> LanguageLoader {
        let cache = HashMap::new();
        LanguageLoader { dir, cache }
    }

    fn load_language(&mut self, name: &str) -> Result<Language, Box<dyn std::error::Error>> {
        let path = self.dir.join(name).join("parser");

        let lib = unsafe { libloading::Library::new(&path) }?;

        let constructor_name = format!("tree_sitter_{}", name);

        let constructor: libloading::Symbol<extern "C" fn() -> Language> =
            unsafe { lib.get(constructor_name.as_bytes()) }?;

        let result = Ok(constructor());

        std::mem::forget(lib); // make sure the library isn't dropped while the language is alive.

        result
    }

    fn get_language(&mut self, name: &str) -> Result<Language, Box<dyn std::error::Error>> {
        let stored_value = self.cache.get(name);
        match stored_value {
            Some(x) => Ok(*x),
            None => {
                let language = self.load_language(name)?;
                self.cache.insert(String::from(name), language);
                return Ok(language);
            }
        }
    }
}

fn init_languages(cli: &Cli) -> HashMap<String, Rc<LanguageBundle>> {
    let query_str = fs::read_to_string(&cli.query).expect("Read query");

    let mut result = HashMap::new();

    let grammar_dir = env::var("GRAMMAR_DIR").expect("Read grammar paths from GRAMMAR_DIR");
    let mut loader = LanguageLoader::new(PathBuf::from(grammar_dir));

    for language_spec in &cli.languages {
        if let [extensions, language_name] =
            &language_spec.split("=").take(2).collect::<Vec<&str>>()[..]
        {
            let language = loader
                .get_language(language_name)
                .expect(format!("Load language: {}", language_name).as_str());

            let query = Query::new(language, &query_str).expect("Construct query");

            let bundle = Rc::new(LanguageBundle { language, query });

            for ext in extensions.split(",") {
                result.insert(ext.to_string(), bundle.clone());
            }
        } else {
            todo!();
        }
    }

    return result;
}

fn main() {
    let args = Cli::parse();

    let langs = init_languages(&args);

    for pattern in args.globs {
        for entry in glob(&pattern).expect("Failed to glob inputs") {
            let entry_path = entry.expect("Read glob entry");

            let extension = entry_path.extension().unwrap().to_str().unwrap();
            let lang = langs
                .get(extension)
                .expect(format!("Getting parser for extension {:?}", extension).as_str());

            let contents = fs::read_to_string(entry_path).expect("Read source file");
            let tree = lang.parse(&contents).expect("Parse source");
            let names = lang.query.capture_names();

            let mut cursor = QueryCursor::new();
            let matches = cursor.matches(&lang.query, tree.root_node(), contents.as_bytes());

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
}
