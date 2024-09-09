use std::{error::Error, path::PathBuf};

use cargo_cite::{keys_to_citations, load_bib, load_style, File};

use hayagriva::{citationberg::IndependentStyle, Library};

use cargo_files_core::{get_target_files, get_targets};
use clap::Parser;
use stderrlog::LogLevelNum;

#[derive(Debug, Parser)]
// Cargo passes "cite" to cargo-cite, so add a hidden argument to capture that.
#[command(
    arg(clap::Arg::new("dummy")
    .value_parser(["cite"])
    .required(false)
    .hide(true))
)]
struct Args {
    /// Bibtex file with citations
    #[clap(short, long, required = true, value_parser = load_bib)]
    bib: Library,
    /// Citation style, e.g. ieee, apa, chicago
    #[clap(short, long, default_value = "mla", value_parser = load_style)]
    style: IndependentStyle,
    /// Cargo manifest location for other crates
    #[clap(short, long = "manifest", conflicts_with = "files")]
    manifest_path: Option<PathBuf>,
    /// Path to file or folder to format. Can be specified multiple times.
    #[clap(short, long = "file", conflicts_with = "manifest_path")]
    files: Vec<PathBuf>,
    /// Verbosity level
    #[clap(short, long, default_value = "false")]
    verbose: bool,
    /// Silence all output
    #[clap(short, long, default_value = "false")]
    quiet: bool,
}

/// A file with comments
fn main() -> Result<(), Box<dyn Error>> {
    // parse using clap
    let args = Args::parse();

    // Initialize logger
    let verbose = if args.verbose {
        LogLevelNum::Info
    } else {
        LogLevelNum::Warn
    };
    stderrlog::new()
        .quiet(args.quiet)
        .verbosity(verbose)
        .init()
        .unwrap();

    // If a manifest or nothing was specified
    let start = std::time::Instant::now();
    let files = if args.files.is_empty() {
        let targets = get_targets(args.manifest_path.as_deref())?;
        targets
            .iter()
            .map(|t| get_target_files(t))
            .flatten()
            .flatten()
            .collect()
    // If specific files were given
    } else {
        let mut found_files = Vec::new();
        for file in args.files {
            if file.is_dir() {
                let glob = file.join("**").join("*.rs");
                for entry in glob::glob(&glob.to_string_lossy())? {
                    let path = entry?;
                    found_files.push(path);
                }
            } else {
                found_files.push(file);
            }
        }
        found_files
    };

    // Cite each file
    for f in files {
        log::info!("Beginning citation for {:?}", f);
        let mut file = File::open(f.clone());
        let citations = keys_to_citations(file.keys(), &args.bib, &args.style);
        // Instead of doing it in the block
        file.cite(&citations);
        file.save();
    }
    println!("Citation took {:?}", start.elapsed());

    Ok(())
}
