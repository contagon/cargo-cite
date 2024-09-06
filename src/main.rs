mod block;
pub use block::Block;

mod file;
pub use file::File;

mod bib;
pub use bib::{keys_to_citations, load_bib, load_style};
use hayagriva::{citationberg::IndependentStyle, Library};

use std::{env, ffi::OsStr, iter};

use cargo_files_core::{get_target_files, get_targets};
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(required = true, long = "bib", value_parser = load_bib)]
    bib: Library,
    #[command(flatten)]
    manifest: clap_cargo::Manifest,
    #[clap(long = "style", default_value = "ieee", value_parser = load_style)]
    style: IndependentStyle,
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
    // TODO: Expand to accept single files and directories via an argument
    // https://github.com/dcchut/cargo-derivefmt/blob/master/cargo-derivefmt/src/main.rs
    let targets = get_targets(cli.manifest.manifest_path.as_deref()).unwrap();
    let files = targets
        .iter()
        .flat_map(|t| get_target_files(t).unwrap())
        .collect::<Vec<_>>();
    println!("{:#?}", files);

    // Cite each file
    for f in files {
        let mut file = File::open(f);
        file.cite(&cli.bib, &cli.style);
        file.save();
    }
}
