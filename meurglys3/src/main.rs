use std::{ffi::OsString, path::PathBuf};

use clap::Parser;
use meurglys3_lib::{self, Package};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    dir: Option<PathBuf>,
    #[arg(short, long)]
    source: Option<OsString>,
    #[arg(short, long)]
    unpack: Option<bool>,
    #[arg(short, long)]
    output: Option<OsString>,
    #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ',')]
    check: Option<Vec<String>>,
}

fn main() {
    let mut pack: Option<Package> = None;
    let args = Args::parse();
    if let Some(true) = args.unpack {
        let Some(file) = args.source.clone() else {
            panic!("source file not specified");
        };
        pack = Some(unpack(
            file,
            args.output.clone().expect("no unpack output specified"),
        ));
    }
    if let Some(check) = args.check {
        return match (args.source, pack) {
            (Some(source), None) => {
                pack = Some(meurglys3_lib::load_package(source).unwrap());
                check_pack(&check, &pack.unwrap())
            }
            (None, Some(pack)) => check_pack(&check, &pack),
            _ => {
                panic!("cannot perform a check without the package path specified")
            }
        };
    }
    if let Some(dir) = args.dir {
        let mut pack = meurglys3_lib::package_dir(dir).expect("Failed to package");
        let out = args.output.unwrap_or("package".into());
        meurglys3_lib::write_package(out, &mut pack).expect("failed to write package");
    }
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
