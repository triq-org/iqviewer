//! Link C API

use std::env;
use std::path::Path;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_target = env::var("TARGET").unwrap();
    let lib_dir = Path::new(&crate_dir).join("lib").join(build_target);
    let lib_dir = lib_dir.to_str().unwrap();
    println!("cargo:rustc-link-search={lib_dir}");
}
