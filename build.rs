use cbindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    match cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_only_target_dependencies(true)
        .with_language(cbindgen::Language::C)
        .generate() {
            Ok(bindings) => {
                bindings.write_to_file("./build/include/trident.h");
            }
            Err(e) => {
                println!("cargo:warning=bindgen error: {}", e);
                panic!("{}", e);
            }
        }
}
