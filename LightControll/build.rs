use std::{env, fs, path::Path};

fn main() {
    embuild::build::CfgArgs::output_propagated("ESP_IDF").unwrap();
    embuild::build::LinkArgs::output_propagated("ESP_IDF").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("security.rs");

    let source = if Path::new("src/security.rs").exists() {
        println!("cargo:rerun-if-changed=src/security.rs");
        "src/security.rs"
    } else {
        println!("cargo:rerun-if-changed=src/protected.rs");
        "src/protected.rs"
    };

    fs::copy(source, dest_path).unwrap();
}
