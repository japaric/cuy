//! Compilation target identification by way of parsing the metadata in
//! object files produced by `rustc`

use std::ffi::OsString;
use std::process::Command;
use std::{env, fs, io};

use object::elf::{self, FileHeader32};
use object::endian::Endianness;
use object::read::elf::{AttributeReader, AttributesSection, ElfFile32, SectionHeader as _};
use object::read::{Object as _, ObjectSection as _};
use temp_dir::TempDir;

/// Identifies the given compilation `target`
pub fn whoarchi(target: &str) -> Result<WhoArchI, Error> {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));

    let temp_dir = TempDir::new()?;
    let temp_dir = temp_dir.path();
    let probe_rs = temp_dir.join("probe.rs");
    fs::write(probe_rs, "#![no_std]")?;

    let output = Command::new(rustc)
        .args([
            "--target",
            target,
            "--emit=obj",
            "--crate-type=rlib",
            "probe.rs",
        ])
        .current_dir(temp_dir)
        .output()?;
    if !output.status.success() {
        return Err(Error::Rustc(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    let probe_o = temp_dir.join("probe.o");
    let bytes = fs::read(probe_o)?;

    let mut aeabi = None;
    if let Ok(file) = ElfFile32::<Endianness>::parse(&*bytes) {
        let endian = file.endian();
        for section in file.sections() {
            let sh_type = section.elf_section_header().sh_type(endian);
            if sh_type == elf::SHT_ARM_ATTRIBUTES {
                let attrs =
                    AttributesSection::<FileHeader32<Endianness>>::new(endian, section.data()?)?;

                for subsection in attrs.subsections()? {
                    let subsection = subsection?;
                    if subsection.vendor() == b"aeabi" {
                        for subsubsection in subsection.subsubsections() {
                            let subsubsection = subsubsection?;

                            assert!(
                                aeabi.is_none(),
                                "TODO: handle multiple aeabi attribute sections"
                            );
                            aeabi = Some(subsubsection.attributes().try_into()?);
                        }
                    }
                }
            }
        }
    }

    Ok(WhoArchI { aeabi })
}

/// AEABI information
pub struct Aeabi {
    arch: Arch,
}

impl Aeabi {
    /// Processor architecture version and architecture profile
    pub fn arch(&self) -> Arch {
        self.arch
    }
}

impl TryFrom<AttributeReader<'_>> for Aeabi {
    type Error = Error;

    fn try_from(mut attributes: AttributeReader<'_>) -> Result<Aeabi, Error> {
        let mut arch = None;
        while let Some(tag) = attributes.read_tag()? {
            // https://github.com/ARM-software/abi-aa/blob/087483c/addenda32/addenda32.rst#public-aeabi-attribute-tags
            match tag {
                // Tag_conformance
                67 => {
                    attributes.read_string()?;
                }
                // Tag_CPU_arch
                6 => {
                    assert!(arch.is_none());
                    arch = Some(Arch::from(attributes.read_integer()?));
                }
                // Tag_CPU_arch_profile
                7 => {
                    attributes.read_integer()?;
                }
                // Tag_CPU_ARM_ISA_use
                8 => {
                    attributes.read_integer()?;
                }
                // Tag_CPU_THUMB_ISA_use
                9 => {
                    attributes.read_integer()?;
                }
                // Tag_FP_arch
                10 => {
                    attributes.read_integer()?;
                }
                // Tag_ABI_PCS_R9_use
                14 => {
                    attributes.read_integer()?;
                }
                // Tag_ABI_PCS_GOT_use
                17 => {
                    attributes.read_integer()?;
                }
                // Tag_ABI_FP_denormal
                20 => {
                    attributes.read_integer()?;
                }
                // Tag_ABI_FP_exceptions
                21 => {
                    attributes.read_integer()?;
                }
                // Tag_ABI_FP_number_model
                23 => {
                    attributes.read_integer()?;
                }
                // Tag_ABI_align_needed
                24 => {
                    attributes.read_integer()?;
                }
                // Tag_ABI_align_preserved
                25 => {
                    attributes.read_integer()?;
                }
                // Tag_ABI_HardFP_use
                27 => {
                    attributes.read_integer()?;
                }
                // Tag_ABI_VFP_args
                28 => {
                    attributes.read_integer()?;
                }
                // Tag_CPU_unaligned_access
                34 => {
                    attributes.read_integer()?;
                }
                // Tag_FP_HP_extension
                36 => {
                    attributes.read_integer()?;
                }
                // Tag_ABI_FP_16bit_format
                38 => {
                    attributes.read_integer()?;
                }
                _ => {
                    todo!("unexpected tag: {}", tag)
                }
            }
        }

        Ok(Aeabi {
            arch: arch.expect("Tag_CPU_arch not found"),
        })
    }
}

/// Processor architecture version and architecture profile
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Arch {
    /// Arm v7, e.g. Cortex-A8, Cortex-M3
    V7,
    /// Arm v7E-M, v7-M with DSP extensions
    V7EM,
    /// Arm v8-M.mainline
    V8MMainline,
}

impl From<u64> for Arch {
    fn from(value: u64) -> Arch {
        match value {
            10 => Arch::V7,
            13 => Arch::V7EM,
            17 => Arch::V8MMainline,
            _ => panic!("unexpected Tag_CPU_arch value: {value}"),
        }
    }
}

/// Error
#[derive(Debug)]
pub enum Error {
    /// IO error
    Io(io::Error),
    /// Object parsing error
    Object(object::Error),
    /// `rustc` compilation error
    Rustc(String),
}

impl From<object::Error> for Error {
    fn from(v: object::Error) -> Self {
        Self::Object(v)
    }
}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        Self::Io(v)
    }
}

/// Compilation target inforamiot
pub struct WhoArchI {
    aeabi: Option<Aeabi>,
}

impl WhoArchI {
    /// AEABI information
    pub fn aeabi(&self) -> Option<&Aeabi> {
        self.aeabi.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbv7m_none_eabi() {
        let aeabi = crate::whoarchi("thumbv7m-none-eabi")
            .unwrap()
            .aeabi
            .unwrap();
        assert_eq!(Arch::V7, aeabi.arch);
    }

    #[test]
    fn thumbv7em_none_eabihf() {
        let aeabi = crate::whoarchi("thumbv7em-none-eabihf")
            .unwrap()
            .aeabi
            .unwrap();
        assert_eq!(Arch::V7EM, aeabi.arch);
    }

    #[test]
    fn thumbv8m_main_none_eabihf() {
        let aeabi = crate::whoarchi("thumbv8m.main-none-eabihf")
            .unwrap()
            .aeabi
            .unwrap();
        assert_eq!(Arch::V8MMainline, aeabi.arch);
    }
}
