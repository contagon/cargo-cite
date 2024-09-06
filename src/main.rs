mod block;
pub use block::Block;

mod file;
pub use file::File;

mod bib;
pub use bib::{keys_to_citations, load_bib, load_style};

use std::{env, ffi::OsStr, iter, path::PathBuf};

use cargo_files_core::get_targets;
use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // Matches citations inline
    static ref RE_CITATION: Regex = Regex::new(r"\[\^@(.*?)\]").unwrap();
    // Matches comments (ie line begins with //! or ///)
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
/// This struct is used to parse a file and extract comments [^@simple]
///
/// [^@simple]: S. Bai, J. Lai, P. Lyu, Y. Cen, B. Wang, and X. Sun, “Graph-Optimisation-Based Self-Calibration Method for IMU/Odometer Using Preintegration Theory,” The Journal of Navigation, vol. 75, no. 3, pp. 594–613, May 2022, doi: 10.1017/S0373463321000722.
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
    let lib = load_bib(cli.bib);
    let style = load_style("ieee");

    for f in files {
        let mut file = File::open(f.path);
        file.cite(&lib, &style);
        file.save();
    }
}
