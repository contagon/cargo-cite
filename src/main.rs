//! This is a large comment!
use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    fs,
    io::{BufRead, BufReader},
    iter,
    path::{Path, PathBuf},
};

use biblatex::Bibliography;
use cargo_files_core::get_targets;
use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // Matches citations inline
    static ref RE_CITATION: Regex = Regex::new(r"\[\^@(.*)\]").unwrap();
    // Matches comments (ie line begins with //! or ////)
    static ref RE_COMMENT: Regex = Regex::new(r"^[ \t]*//[/!]").unwrap();
    // Matches footnotes (ie line begins with /// [^@citekey;]:)
    static ref RE_FOOTNOTE: Regex = Regex::new(r"^[ \t]*//[/!]\s*\[\^@(.*)\]:").unwrap();
}

#[derive(Debug, Parser)]
struct Cli {
    #[clap(required = true, long = "bib")]
    bib: PathBuf,
    #[command(flatten)]
    manifest: clap_cargo::Manifest,
}

fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = fs::File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

#[derive(Debug, Clone)]
struct Comment {
    start: usize,
    end: usize,
}

impl Comment {
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Fast check if the comment has references
    pub fn has_references(&self, lines: &[String]) -> bool {
        for line in &lines[self.start..self.end] {
            if line.contains("[^@") {
                return true;
            }
        }
        false
    }

    /// A file with comments
    ///
    /// This struct is used to parse a file and extract comments [^@simple]
    ///
    /// [^@simple]: This is a simple reference
    pub fn references(&self, lines: &[String]) -> HashSet<String> {
        let mut refs = HashSet::new();
        for line in &lines[self.start..self.end] {
            for cite in RE_CITATION.captures_iter(line) {
                refs.insert(cite[1].to_string());
            }
        }
        refs
    }
}

#[derive(Debug, Clone)]
struct File {
    lines: Vec<String>,
    comments: Vec<Comment>,
}

impl File {
    pub fn open(file: impl AsRef<Path>) -> Self {
        let lines = lines_from_file(file);
        let mut comments = Vec::new();

        let mut start = None;
        for (i, line) in lines.iter().enumerate() {
            if RE_COMMENT.is_match(line) {
                match start {
                    None => start = Some(i),
                    Some(_) => {}
                }
            } else if start.is_some() {
                comments.push(Comment {
                    start: start.unwrap(),
                    // TODO: End on i - 1?
                    end: i,
                });
                start = None;
            }
        }

        Self { lines, comments }
    }

    // TODO: Check if footnote is already included
    pub fn references(&self) -> HashSet<String> {
        self.comments
            .iter()
            .flat_map(|c| c.references(&self.lines))
            .collect()
    }
}

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
    println!("{:?}", cli);

    // Get all file names to parse
    // TODO: Expand to accept single files
    // https://github.com/dcchut/cargo-derivefmt/blob/master/cargo-derivefmt/src/main.rs
    let files = get_targets(cli.manifest.manifest_path.as_deref()).unwrap();
    println!("{:?}", files);

    // load bib file
    let bib = std::fs::read_to_string(&cli.bib).expect("Could not read bib file");
    let _bib = Bibliography::parse(&bib).unwrap();

    for f in files {
        let file = File::open(f.path);
        println!("{:?}", file.references());
    }
}
