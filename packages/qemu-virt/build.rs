//! Adds linker script to linker search path

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write as _;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    const SCRIPT: &str = "memory.ld";

    let out_dir = env::var("OUT_DIR")?;
    File::create(Path::new(&out_dir).join(SCRIPT))?.write_all(include_bytes!("memory.ld"))?;

    println!("cargo::rustc-link-search={out_dir}");
    println!("cargo:rustc-link-arg=-T{SCRIPT}");

    Ok(())
}
