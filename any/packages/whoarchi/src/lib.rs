//! Compilation target identification by way of parsing the metadata in object files produced by
//! `rustc`
//!
//! # References
//! - `AEABI`: Addenda to, and Errata in, the ABI for the Arm® Architecture 2025Q4 [1]
//!
//! [1]: https://github.com/ARM-software/abi-aa/blob/087483c/addenda32/addenda32.rst

#![allow(non_camel_case_types)]

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

                            assert!(aeabi.is_none(), "found multiple aeabi attribute sections");
                            aeabi = Some(subsubsection.attributes().try_into()?);
                        }
                    }
                }
            }
        }
    }

    Ok(WhoArchI { aeabi })
}

/// Compilation target information
#[derive(Clone, Debug, PartialEq)]
pub struct WhoArchI {
    aeabi: Option<Aeabi>,
}

impl WhoArchI {
    /// AEABI information
    pub fn aeabi(&self) -> Option<&Aeabi> {
        self.aeabi.as_ref()
    }
}

/// AEABI information
#[derive(Clone, Debug, PartialEq)]
pub struct Aeabi {
    abi_vfp_args: Option<ABI_VFP_args>,
    cpu_arch: CPU_arch,
    cpu_arch_profile: Option<CPU_arch_profile>,
    fp_arch: Option<FP_arch>,
}

impl Aeabi {
    /// ABI used to pass FP parameters/results, if present
    ///
    /// The presence of this attribute could indicate that the target is either "hard float" or "soft fp"
    pub fn abi_vfp_args(&self) -> Option<ABI_VFP_args> {
        self.abi_vfp_args
    }

    /// Processor architecture version and architecture profile
    pub fn cpu_arch(&self) -> CPU_arch {
        self.cpu_arch
    }

    /// Architecture profile, if present
    pub fn cpu_arch_profile(&self) -> Option<CPU_arch_profile> {
        self.cpu_arch_profile
    }

    /// ⚠️ FP architecture, if present
    ///
    /// ⚠️ The presence of this attribute does NOT imply that the target is a "hard float" target. Instead, use
    /// `abi_vfp_args`
    pub fn fp_arch(&self) -> Option<FP_arch> {
        self.fp_arch
    }
}

impl TryFrom<AttributeReader<'_>> for Aeabi {
    type Error = Error;

    fn try_from(mut attributes: AttributeReader<'_>) -> Result<Aeabi, Error> {
        #![allow(non_upper_case_globals)]

        // [AEABI/3.3]
        const Tag_ABI_FP_16bit_format: u64 = 38;
        const Tag_ABI_FP_denormal: u64 = 20;
        const Tag_ABI_FP_exceptions: u64 = 21;
        const Tag_ABI_FP_number_model: u64 = 23;
        const Tag_ABI_HardFP_use: u64 = 27;
        const Tag_ABI_PCS_GOT_use: u64 = 17;
        const Tag_ABI_PCS_R9_use: u64 = 14;
        const Tag_ABI_VFP_args: u64 = 28;
        const Tag_ABI_align_needed: u64 = 24;
        const Tag_ABI_align_preserved: u64 = 25;
        const Tag_ARM_ISA_use: u64 = 8;
        const Tag_CPU_arch: u64 = 6;
        const Tag_CPU_arch_profile: u64 = 7;
        const Tag_CPU_unaligned_access: u64 = 34;
        const Tag_FP_HP_extension: u64 = 36;
        const Tag_FP_arch: u64 = 10;
        const Tag_MPextension_use: u64 = 42;
        const Tag_THUMB_ISA_use: u64 = 9;
        const Tag_Virtualization_use: u64 = 68;
        const Tag_conformance: u64 = 67;

        let mut abi_vfp_args = None;
        let mut cpu_arch = None;
        let mut cpu_arch_profile = None;
        let mut fp_arch = None;

        while let Some(tag) = attributes.read_tag()? {
            match tag {
                Tag_ABI_VFP_args => {
                    assert!(
                        abi_vfp_args.is_none(),
                        "Tag_ABI_VFP_args appeared more than once"
                    );

                    abi_vfp_args = Some(ABI_VFP_args::from(attributes.read_integer()?));
                }

                Tag_CPU_arch => {
                    assert!(cpu_arch.is_none(), "Tag_CPU_arch appeared more than once");

                    cpu_arch = Some(CPU_arch::from(attributes.read_integer()?));
                }

                Tag_CPU_arch_profile => {
                    assert!(
                        cpu_arch_profile.is_none(),
                        "Tag_CPU_arch_profile appeared more than once"
                    );

                    cpu_arch_profile = Some(CPU_arch_profile::from(attributes.read_integer()?));
                }

                Tag_FP_arch => {
                    assert!(fp_arch.is_none(), "Tag_FP_arch appeared more than once");

                    fp_arch = Some(FP_arch::from(attributes.read_integer()?));
                }

                Tag_ABI_FP_16bit_format => _ = attributes.read_integer()?,
                Tag_ABI_FP_denormal => _ = attributes.read_integer()?,
                Tag_ABI_FP_exceptions => _ = attributes.read_integer()?,
                Tag_ABI_FP_number_model => _ = attributes.read_integer()?,
                Tag_ABI_HardFP_use => _ = attributes.read_integer()?,
                Tag_ABI_PCS_GOT_use => _ = attributes.read_integer()?,
                Tag_ABI_PCS_R9_use => _ = attributes.read_integer()?,
                Tag_ABI_align_needed => _ = attributes.read_integer()?,
                Tag_ABI_align_preserved => _ = attributes.read_integer()?,
                Tag_ARM_ISA_use => _ = attributes.read_integer()?,
                Tag_CPU_unaligned_access => _ = attributes.read_integer()?,
                Tag_FP_HP_extension => _ = attributes.read_integer()?,
                Tag_MPextension_use => _ = attributes.read_integer()?,
                Tag_THUMB_ISA_use => _ = attributes.read_integer()?,
                Tag_Virtualization_use => _ = attributes.read_integer()?,
                Tag_conformance => _ = attributes.read_string()?,

                _ => panic!("unsupported tag value: {tag}"),
            }
        }

        let cpu_arch = cpu_arch.expect("Tag_CPU_arch not found");
        Ok(Aeabi {
            abi_vfp_args,
            cpu_arch,
            cpu_arch_profile,
            fp_arch,
        })
    }
}

/// Which ABI is used to pass FP parameters/result
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum ABI_VFP_args {
    /// AAPCS, VFP variant. AKA "hard float"
    VFP,
}

impl From<u64> for ABI_VFP_args {
    fn from(value: u64) -> Self {
        // [AEABI/3.3.6.2]
        match value {
            1 => ABI_VFP_args::VFP,

            _ => panic!("unsupported Tag_ABI_VFP_args value: {value}"),
        }
    }
}

/// Processor architecture version and architecture profile
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CPU_arch {
    /// Arm v6S-M, v6-M with the System extensions
    v6S_M,
    /// ⚠️ Arm v7, e.g. Cortex-A8, Cortex-M3
    ///
    /// ⚠️ This variant does not specify a profile so you need to check `CPU_arch_profile`
    v7,
    /// Arm v7E-M, v7-M with DSP extensions
    v7E_M,
    /// Arm v8-R
    v8_R,
    /// Arm v8-M.baseline
    v8_M_baseline,
    /// Arm v8-M.mainline
    v8_M_mainline,
}

impl From<u64> for CPU_arch {
    fn from(value: u64) -> CPU_arch {
        // [AEABI/3.3.5.2]
        match value {
            10 => CPU_arch::v7,
            12 => CPU_arch::v6S_M,
            13 => CPU_arch::v7E_M,
            15 => CPU_arch::v8_R,
            16 => CPU_arch::v8_M_baseline,
            17 => CPU_arch::v8_M_mainline,

            _ => panic!("not yet supported Tag_CPU_arch value: {value}"),
        }
    }
}

/// Architecture profile
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CPU_arch_profile {
    /// Application profile
    A,
    /// Realtime profile
    R,
    /// Microcontroller profile
    M,
}

impl From<u64> for CPU_arch_profile {
    fn from(value: u64) -> Self {
        // [AEABI/3.3.5.2]
        match value.try_into() {
            Ok(b'A') => CPU_arch_profile::A,
            Ok(b'R') => CPU_arch_profile::R,
            Ok(b'M') => CPU_arch_profile::M,

            _ => panic!("unsupported Tag_CPU_arch_profile value: {value}"),
        }
    }
}

/// FP ISA
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum FP_arch {
    /// Use of the v3 FP ISA was permitted (implies use of the v2 FP ISA)
    VFPv3,
    /// Use of the v3 FP ISA was permitted, but only citing registers D0-D15, S0-S31
    VFPv3_D16,
    /// Use of the v4 FP ISA was permitted, but only citing registers D0-D15, S0-S31
    VFPv4_D16,
    /// Use of the Arm v8-A FP ISA was permitted, but only citing registers D0-D15, S0-S31
    FP_D16,
}

impl From<u64> for FP_arch {
    fn from(value: u64) -> Self {
        // [AEABI/3.3.5.2]
        match value {
            3 => FP_arch::VFPv3,
            4 => FP_arch::VFPv3_D16,
            6 => FP_arch::VFPv4_D16,
            8 => FP_arch::FP_D16,

            _ => panic!("unsupported Tag_FP_arch: {value}"),
        }
    }
}

/// Error
#[derive(Debug)]
#[non_exhaustive]
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

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn all() {
        let cases = [
            ("aarch64-unknown-none", WhoArchI { aeabi: None }),
            ("aarch64-unknown-none-softfloat", WhoArchI { aeabi: None }),
            (
                "armv7a-none-eabi",
                WhoArchI {
                    aeabi: Some(Aeabi {
                        abi_vfp_args: None,
                        cpu_arch: CPU_arch::v7,
                        cpu_arch_profile: Some(CPU_arch_profile::A),
                        fp_arch: Some(FP_arch::VFPv3),
                    }),
                },
            ),
            (
                "armv7a-none-eabihf",
                WhoArchI {
                    aeabi: Some(Aeabi {
                        abi_vfp_args: Some(ABI_VFP_args::VFP),
                        cpu_arch: CPU_arch::v7,
                        cpu_arch_profile: Some(CPU_arch_profile::A),
                        fp_arch: Some(FP_arch::VFPv3),
                    }),
                },
            ),
            (
                "armv7r-none-eabi",
                WhoArchI {
                    aeabi: Some(Aeabi {
                        abi_vfp_args: None,
                        cpu_arch: CPU_arch::v7,
                        cpu_arch_profile: Some(CPU_arch_profile::R),
                        fp_arch: None,
                    }),
                },
            ),
            (
                "armv7r-none-eabihf",
                WhoArchI {
                    aeabi: Some(Aeabi {
                        abi_vfp_args: Some(ABI_VFP_args::VFP),
                        cpu_arch: CPU_arch::v7,
                        cpu_arch_profile: Some(CPU_arch_profile::R),
                        fp_arch: Some(FP_arch::VFPv3_D16),
                    }),
                },
            ),
            (
                "armv8r-none-eabihf",
                WhoArchI {
                    aeabi: Some(Aeabi {
                        abi_vfp_args: Some(ABI_VFP_args::VFP),
                        cpu_arch: CPU_arch::v8_R,
                        cpu_arch_profile: Some(CPU_arch_profile::R),
                        fp_arch: Some(FP_arch::FP_D16),
                    }),
                },
            ),
            (
                "thumbv6m-none-eabi",
                WhoArchI {
                    aeabi: Some(Aeabi {
                        abi_vfp_args: None,
                        cpu_arch: CPU_arch::v6S_M,
                        cpu_arch_profile: Some(CPU_arch_profile::M),
                        fp_arch: None,
                    }),
                },
            ),
            (
                "thumbv7em-none-eabi",
                WhoArchI {
                    aeabi: Some(Aeabi {
                        abi_vfp_args: None,
                        cpu_arch: CPU_arch::v7E_M,
                        cpu_arch_profile: Some(CPU_arch_profile::M),
                        fp_arch: None,
                    }),
                },
            ),
            (
                "thumbv7em-none-eabihf",
                WhoArchI {
                    aeabi: Some(Aeabi {
                        abi_vfp_args: Some(ABI_VFP_args::VFP),
                        cpu_arch: CPU_arch::v7E_M,
                        cpu_arch_profile: Some(CPU_arch_profile::M),
                        fp_arch: Some(FP_arch::VFPv4_D16),
                    }),
                },
            ),
            (
                "thumbv7m-none-eabi",
                WhoArchI {
                    aeabi: Some(Aeabi {
                        abi_vfp_args: None,
                        cpu_arch: CPU_arch::v7,
                        cpu_arch_profile: Some(CPU_arch_profile::M),
                        fp_arch: None,
                    }),
                },
            ),
            (
                "thumbv8m.base-none-eabi",
                WhoArchI {
                    aeabi: Some(Aeabi {
                        abi_vfp_args: None,
                        cpu_arch: CPU_arch::v8_M_baseline,
                        cpu_arch_profile: Some(CPU_arch_profile::M),
                        fp_arch: None,
                    }),
                },
            ),
            (
                "thumbv8m.main-none-eabi",
                WhoArchI {
                    aeabi: Some(Aeabi {
                        abi_vfp_args: None,
                        cpu_arch: CPU_arch::v8_M_mainline,
                        cpu_arch_profile: Some(CPU_arch_profile::M),
                        fp_arch: None,
                    }),
                },
            ),
            (
                "thumbv8m.main-none-eabihf",
                WhoArchI {
                    aeabi: Some(Aeabi {
                        abi_vfp_args: Some(ABI_VFP_args::VFP),
                        cpu_arch: CPU_arch::v8_M_mainline,
                        cpu_arch_profile: Some(CPU_arch_profile::M),
                        fp_arch: Some(FP_arch::FP_D16),
                    }),
                },
            ),
        ];

        for (target, expected) in cases {
            let got = crate::whoarchi(target)
                .unwrap_or_else(|e| panic!("could not identify {target} due to {e:?}"));
            assert_eq!(expected, got, "{target}")
        }
    }
}
