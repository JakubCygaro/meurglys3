use std::path::PathBuf;

use clap::Parser;
use meurglys3_lib::{self, Package};

#[derive(Parser, Debug)]
#[command(version)]
#[command(about = "A tar-like packaging utility")]
#[command(name = "Meurglys3")]
#[command(
    long_about = "Packages whole directories (including subdirectories) into a single .m3pkg file while preserving the directory structure for later unpacking."
)]
#[command(author = "Adam Papieros")]
struct Args {
    #[command(subcommand)]
    target: Target,
    // #[arg(short, long)]
    // dir: Option<PathBuf>,
    // #[arg(short, long)]
    // source: Option<PathBuf>,
    // #[arg(short, long)]
    // unpack: Option<bool>,
    // #[arg(short, long)]
    // output: Option<PathBuf>,
    // #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ',')]
    // check: Option<Vec<String>>,
}

#[derive(clap::Subcommand, Debug)]
enum Target {
    #[command(about = "Package a directory", long_about = None)]
    Pack {
        #[arg(help = "source directory")]
        dir: PathBuf,
        #[arg(help = "output file name")]
        out: PathBuf,
    },
    #[command(about = "Unpackage a directory", long_about = None)]
    Unpack {
        #[arg(help = "source .m3pkg file")]
        dir: PathBuf,
        #[arg(help = "output directory path")]
        out: PathBuf,
    },
    #[command(about = "Check wether a package contains a file", long_about = None)]
    Check {
        #[arg(help = "source .m3pkg file")]
        dir: PathBuf,
        #[arg(
            help = "a comma separated list of file paths to check if they are contained in the package"
        )]
        #[clap(value_parser, num_args = 1.., value_delimiter = ',')]
        check: Vec<String>,
    },
}

fn main() {
    let args = Args::parse();

    match args.target {
        Target::Pack { dir, out } => {
            let mut pack = meurglys3_lib::package_dir(dir).expect("Failed to package");
            meurglys3_lib::write_package(out, &mut pack).expect("failed to write package");
        }
        Target::Unpack { dir, out } => {
            unpack(dir, out);
        }
        Target::Check { dir, check } => {
            let pack = meurglys3_lib::load_package(dir.clone()).unwrap_or_else(|_| panic!(
                "could not load package at `{}`",
                dir.to_str().unwrap_or_default()
            ));
            check_pack(&check, &pack)
        }
    };
}
fn unpack(source: PathBuf, dest: PathBuf) -> Package {
    let pack = meurglys3_lib::load_package(source).expect("unpack failed");
    //println!("{:?}", pack)
    meurglys3_lib::unpack_to_dir(dest, &pack).unwrap();
    pack
}
fn check_pack(names: &Vec<String>, pack: &Package) {
    for n in names {
        if pack.has(n) {
            println!("The package contains `{n}`")
        } else {
            println!("The package does not contain `{n}`")
        }
    }
}
