use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let linker = manifest_dir.join("linker.ld");
    println!("cargo:rustc-link-arg=-T{}", linker.display());
}
