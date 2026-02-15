use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Generate Gekko (PowerPC) decoder
    chipi::generate("dsl/gekko.chipi", out_dir.join("gekko.rs").to_str().unwrap())
        .expect("failed to generate gekko decoder");
    println!("cargo:rerun-if-changed=dsl/gekko.chipi");

    // Generate DSP decoder
    chipi::generate("dsl/dsp.chipi", out_dir.join("dsp.rs").to_str().unwrap())
        .expect("failed to generate dsp decoder");
    println!("cargo:rerun-if-changed=dsl/dsp.chipi");
}
