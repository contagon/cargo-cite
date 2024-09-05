mod line;
pub use line::Block;

mod file;
pub use file::File;

mod bib;

use std::{env, ffi::OsStr, iter, path::PathBuf};

use biblatex::Bibliography;
use cargo_files_core::get_targets;
use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // Matches citations inline
    static ref RE_CITATION: Regex = Regex::new(r"\[\^@(.*?)\]").unwrap();
    // Matches comments (ie line begins with //! or ////)
    static ref RE_COMMENT: Regex = Regex::new(r"^[ \t]*//[/!]").unwrap();
    // Matches footnotes (ie line begins with /// [^@citekey]:)
    static ref RE_FOOTNOTE: Regex = Regex::new(r"^[ \t]*//[/!]\s*\[\^@(.*?)\]:").unwrap();
}

#[derive(Debug, Parser)]
struct Cli {
    #[clap(required = true, long = "bib")]
    bib: PathBuf,
    #[command(flatten)]
    manifest: clap_cargo::Manifest,
}

/// A file with comments
///
/// This struct is used to parse[^@test] a file and extract comments [^@simple]
///
/// [^@test]: This is a test reference
/// [^@simple]: This is a simple reference
fn main() {
    // Skip cargo & binary name
    let mut args = env::args_os().peekable();
    let binary_name = args.next().expect("Should have binary name");
    if args.peek().map(OsStr::new) == Some(OsStr::new("cite")) {
        args.next();
    }

    // parse using clap
    let args = iter::once(binary_name).chain(args).collect::<Vec<_>>();
    let cli = Cli::parse_from(args);
    // println!("{:?}", cli);

    // Get all file names to parse
    // TODO: Expand to accept single files
    // https://github.com/dcchut/cargo-derivefmt/blob/master/cargo-derivefmt/src/main.rs
    let files = get_targets(cli.manifest.manifest_path.as_deref()).unwrap();
    // println!("{:?}", files);

    // load bib file
    let bib = std::fs::read_to_string(&cli.bib).expect("Could not read bib file");
    let _bib = Bibliography::parse(&bib).unwrap();

    for f in files {
        let mut file = File::open(f.path);
        file.cite(&_bib);
        file.save();
    }
}
