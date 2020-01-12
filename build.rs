use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.contains("pc-windows") {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let mut lib_dir = manifest_dir.clone();
        lib_dir.push("lib");
        lib_dir.push(&target);
        lib_dir.push("lib");
        println!("cargo:rustc-link-search=all={}", lib_dir.display());
    }
}
