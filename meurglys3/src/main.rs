use std::{ffi::OsString, path::PathBuf};

use clap::Parser;
use meurglys3_lib::{self, Package};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    target: Target,
    // #[arg(short, long)]
    // dir: Option<PathBuf>,
    // #[arg(short, long)]
    // source: Option<OsString>,
    // #[arg(short, long)]
    // unpack: Option<bool>,
    // #[arg(short, long)]
    // output: Option<OsString>,
    // #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ',')]
    // check: Option<Vec<String>>,
}

#[derive(clap::Subcommand, Debug)]
enum Target {
    Pack {
        dir: PathBuf,
        out: OsString
    },
    Unpack{
        dir: OsString,
        out: OsString
    },
    Check {
        dir: OsString,
        #[clap(value_parser, num_args = 1.., value_delimiter = ',')]
        check: Vec<String>
    }
}

fn main() {
    let mut pack: Option<Package> = None;
    let args = Args::parse();

    match args.target {
        Target::Pack { dir, out } => {
            let mut pack = meurglys3_lib::package_dir(dir).expect("Failed to package");
            meurglys3_lib::write_package(out, &mut pack).expect("failed to write package");
        },
        Target::Unpack { dir, out } => {
            pack = Some(unpack(
                dir,
                out,
            ));
        },
        Target::Check { dir, check } => {
            pack = Some(meurglys3_lib::load_package(dir.clone()).expect(&format!("could not load package at `{}`", dir.to_str().unwrap_or_default())));
            check_pack(&check, &pack.unwrap())
        }
    };
}
fn unpack(source: OsString, dest: OsString) -> Package {
    let pack = meurglys3_lib::load_package(source).expect("unpack failed");
    //println!("{:?}", pack)
    meurglys3_lib::unpack_to_dir(dest.into_string().unwrap(), &pack).unwrap();
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
