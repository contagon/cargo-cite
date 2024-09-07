use std::{error::Error, path::PathBuf};

use cargo_cite::{load_bib, load_style, File};

use hayagriva::{citationberg::IndependentStyle, Library};

use cargo_files_core::{get_target_files, get_targets};
use clap::{ArgAction, Parser};

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
    /// Citation style, ie ieee, apa, chicago
    #[clap(short, long, default_value = "mla", value_parser = load_style)]
    style: IndependentStyle,
    /// Cargo manifest location for other crates
    #[clap(short, long = "manifest", conflicts_with = "files")]
    manifest_path: Option<PathBuf>,
    /// Path to file or folder to format. Can be specified multiple times.
    #[clap(short, long = "file", conflicts_with = "manifest_path")]
    files: Vec<PathBuf>,
    /// Verbosity level
    // TODO: Fix verbosity
    // TODO: Fill out README
    // TODO: flamegraph check for performance (I bet it's hayagravia)
    #[clap(short, long, action = ArgAction::Count)]
    verbose: u8,
    /// Silence all output
    #[clap(short, long, action)]
    quiet: bool,
}

/// A file with comments
fn main() -> Result<(), Box<dyn Error>> {
    // parse using clap
    let args = Args::parse();

    // Initialize logger
    let verbose = if args.verbose == 0 { 1 } else { args.verbose };
    stderrlog::new()
        .quiet(args.quiet)
        .verbosity(verbose as usize)
        .init()
        .unwrap();

    // If a manifest or nothing was specified
    log::debug!("Gathering files...");
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
        log::debug!("Parsing file: {:?}", f);
        let mut file = File::open(f.clone());
        log::debug!("Citing file: {:?}", f);
        file.cite(&args.bib, &args.style);
        log::debug!("Saving file: {:?}", f);
        file.save();
    }

    Ok(())
}
