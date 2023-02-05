mod format;

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use clap::Parser as ClapParser;
use glob::glob;
use tree_sitter::{Language, Parser, Query, QueryCursor, Tree};

#[derive(clap::Parser)]
struct Cli {
    #[arg(short, long)]
    query: PathBuf,

    #[arg(short, long)]
    languages: Vec<String>,

    #[arg(short, long)]
    exclude: Vec<String>,

    #[arg(short, long, value_enum, default_value_t = Format::Terse)]
    format: Format,

    globs: Vec<String>,
}

#[derive(clap::ValueEnum, Clone)]
pub enum Format {
    Terse,
    Verbose,
    Snippet,
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

    fn new_with_default_path() -> LanguageLoader {
        let grammar_dir = env::var("GRAMMAR_DIR").expect("Read grammar paths from GRAMMAR_DIR");
        Self::new(PathBuf::from(grammar_dir))
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
                Ok(language)
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn get_language(name: &str, query: &impl AsRef<Path>) -> Rc<LanguageBundle> {
    let mut loader = LanguageLoader::new_with_default_path();
    let query_str = fs::read_to_string(query).expect("Read query");
    let language = loader.get_language(name).unwrap();
    let query = Query::new(language, &query_str).expect("Construct query");
    Rc::new(LanguageBundle { language, query })
}

pub(crate) fn init_languages(
    query: &Path,
    languages: impl IntoIterator<Item = impl Borrow<str>>,
) -> HashMap<String, Rc<LanguageBundle>> {
    let query_str = fs::read_to_string(query)
        .unwrap_or_else(|_| panic!("Reading query from '{}'", query.display()));

    let mut result = HashMap::new();

    let mut loader = LanguageLoader::new_with_default_path();

    for language_spec in languages {
        let language_spec = language_spec.borrow();
        if let [extensions, language_name] =
            &language_spec.split('=').take(2).collect::<Vec<&str>>()[..]
        {
            let language = loader
                .get_language(language_name)
                .unwrap_or_else(|_| panic!("Load language: {}", language_name));

            let query = Query::new(language, &query_str).expect("Construct query");

            let bundle = Rc::new(LanguageBundle { language, query });

            for ext in extensions.split(',') {
                result.insert(ext.to_string(), bundle.clone());
            }
        } else {
            todo!();
        }
    }

    result
}

pub(crate) fn process_file(
    writer: &mut impl Write,
    formatter: &impl format::Formatter,
    path: &impl AsRef<Path>,
    lang: &LanguageBundle,
) -> io::Result<()> {
    let contents = fs::read_to_string(path).expect("Read source file");
    let tree = lang.parse(&contents).expect("Parse source");

    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&lang.query, tree.root_node(), contents.as_bytes());

    formatter.emit_matches(writer, &lang.query, &contents, path, matches)?;
    Ok(())
}

fn main() {
    let args = Cli::parse();

    let langs = init_languages(&args.query, args.languages);

    let excluded = {
        let mut excluded_files = HashSet::new();
        for path in args.exclude {
            let canonical_path = std::fs::canonicalize(path).expect("Canonicalize excluded file");
            excluded_files.insert(canonical_path);
        }
        move |path: &PathBuf| {
            let canonical_path = std::fs::canonicalize(path).expect("Canonicalize input file");
            excluded_files.contains(&canonical_path)
        }
    };

    let mut output = io::stdout();

    type ProcessFile = dyn FnMut(&Path, &LanguageBundle) -> io::Result<()>;
    let mut f: Box<ProcessFile> = match args.format {
        Format::Terse => Box::new(move |path: &Path, lang: &LanguageBundle| {
            process_file(&mut output, &crate::format::terse::Terse {}, &path, lang)
        }),
        Format::Verbose => Box::new(move |path: &Path, lang: &LanguageBundle| {
            process_file(&mut output, &crate::format::terse::Terse {}, &path, lang)
        }),
        Format::Snippet => Box::new(move |path: &Path, lang: &LanguageBundle| {
            process_file(
                &mut output,
                &crate::format::snippet::SnippetFormatter {},
                &path,
                lang,
            )
        }),
    };

    for pattern in args.globs {
        for entry in glob(&pattern).expect("Failed to glob inputs") {
            let entry_path = entry.expect("Read glob entry");

            if excluded(&entry_path) {
                continue;
            }

            let extension = entry_path.extension().unwrap().to_str().unwrap();
            let lang = langs
                .get(extension)
                .unwrap_or_else(|| panic!("Getting parser for extension {:?}", extension));

            f(entry_path.as_path(), lang).unwrap();
        }
    }
}
