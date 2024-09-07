use std::{error::Error, path::PathBuf};

use cargo_cite::{load_bib, load_style, File};

use hayagriva::{citationberg::IndependentStyle, Library};

use cargo_files_core::{get_target_files, get_targets};
use clap::Parser;

#[derive(Debug, Parser)]
// Cargo passes "cite" to cargo-cite, so add a hidden argument to capture that.
#[command(
    arg(clap::Arg::new("dummy")
    .value_parser(["cite"])
    .required(false)
    .hide(true))
)]
struct Args {
    // Bibliography file
    #[clap(required = true, long = "bib", value_parser = load_bib)]
    bib: Library,
    // Citation style
    #[clap(long = "style", default_value = "ieee", value_parser = load_style)]
    style: IndependentStyle,
    // Cargo manifest location for other crates
    #[clap(long, conflicts_with = "files")]
    manifest_path: Option<PathBuf>,
    /// Path to file or folder to format.  Can be specified multiple times.
    #[clap(short, long = "file", conflicts_with = "manifest_path")]
    files: Vec<PathBuf>,
}

/// A file with comments
fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    stderrlog::new().verbosity(3).init().unwrap();

    // parse using clap
    let args = Args::parse();

    // If a manifest or nothing was specified
    let files = if args.files.is_empty() {
        let targets = get_targets(args.manifest_path.as_deref())?;
        targets
            .iter()
            .map(|t| get_target_files(t))
            .flatten()
            .flatten()
            .collect()
    // If specific files were specified
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
        let mut file = File::open(f);
        file.cite(&args.bib, &args.style);
        file.save();
    }

    Ok(())
}
