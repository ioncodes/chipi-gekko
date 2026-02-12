use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    chipi::generate("dsl/gekko.chipi", out_dir.join("gekko.rs").to_str().unwrap())
        .expect("failed to generate decoder");
    println!("cargo:rerun-if-changed=dsl/gekko.chipi");
}
