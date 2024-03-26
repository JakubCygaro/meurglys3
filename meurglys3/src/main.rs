use std::{ffi::OsString, path::PathBuf};

use clap::Parser;
use meurglys3_lib;

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
}

fn main() {
    let args = Args::parse();
    if let Some(true) = args.unpack {
        let Some(file) = args.source else {
            panic!("source file not specified");
        };
        return unpack(file, args.output.expect("no unpack output specified"));
    }
    let dir = args.dir.expect("pack dir not specified");
    let mut pack = meurglys3_lib::package_dir(dir).expect("Failed to package");
    let out = args.output.unwrap_or("package".into());
    meurglys3_lib::write_package(out, &mut pack).expect("failed to write package");
}
fn unpack(source: OsString, dest: OsString) {
    let pack = meurglys3_lib::load_package(source).expect("unpack failed");
    //println!("{:?}", pack)
    meurglys3_lib::unpack_to_dir(dest.into_string().unwrap(), &pack).unwrap();
}
