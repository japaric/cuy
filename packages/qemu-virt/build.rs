//! Adds linker script to linker search path

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write as _;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    provide_linker_script_to_downstream()?;
    set_linker_scripts_for_examples();

    Ok(())
}

fn set_linker_scripts_for_examples() {
    println!("cargo:rustc-link-arg=-Tlayout.ld"); // from rt crate
    println!("cargo:rustc-link-arg=-Tmemory.ld"); // from this crate
}

fn provide_linker_script_to_downstream() -> Result<(), Box<dyn Error>> {
    const SCRIPT: &str = "memory.ld";

    let out_dir = env::var("OUT_DIR")?;
    File::create(Path::new(&out_dir).join(SCRIPT))?.write_all(include_bytes!("src/memory.ld"))?;

    println!("cargo::rustc-link-search={out_dir}");

    Ok(())
}
