//! Adds linker script to linker search path

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write as _;
use std::path::Path;

use whoarchi::{CPU_arch, CPU_arch_profile};

fn main() -> Result<(), Box<dyn Error>> {
    check_compilation_target()?;
    provide_linker_script_to_downstream()?;

    Ok(())
}

fn check_compilation_target() -> Result<(), Box<dyn Error>> {
    let target = env::var("TARGET").expect("$TARGET was not set by Cargo");
    let who_arch_i = whoarchi::id_target(&target).expect("could not identify $TARGET");
    let aeabi = who_arch_i
        .aeabi()
        .expect("only AEABI targets are supported");
    eprintln!("{aeabi:#?}");
    assert_eq!(
        Some(CPU_arch_profile::M),
        aeabi.cpu_arch_profile(),
        "only M profile devices are supported"
    );
    match aeabi.cpu_arch() {
        // OK
        CPU_arch::v7 | CPU_arch::v7E_M | CPU_arch::v8_M_mainline => {}

        CPU_arch::v6S_M => panic!("Armv6-M does not support VTOR; this CPU is not supported"),
        CPU_arch::v8_M_baseline => {
            panic!("Armv8-M.baseline does not support VTOR; this CPU is not supported")
        }
        // R-profile
        CPU_arch::v8_R => unreachable!(),

        cpu_arch => todo!("unknown CPU architecture: {cpu_arch:?}; update this logic"),
    }

    //
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
