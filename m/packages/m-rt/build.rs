//! Adds linker script to linker search path

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write as _;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    provide_linker_script_to_downstream()?;

    Ok(())
}

fn provide_linker_script_to_downstream() -> Result<(), Box<dyn Error>> {
    const SCRIPT: &str = "layout.ld";

    let out_dir = env::var("OUT_DIR")?;
    File::create(Path::new(&out_dir).join(SCRIPT))?.write_all(include_bytes!("src/layout.ld"))?;

    println!("cargo::rustc-link-search={out_dir}");
    println!("cargo::rerun-if-changed={SCRIPT}");

    Ok(())
}
