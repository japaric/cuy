{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell {
  buildInputs = [
    # Rust toolchain manager
    rustup

    # saner Makefiles
    just

    # needed to build minicov
    glibc_multi # 32-bit headers for GNU libc
    clang

    # needed to make codecov reports
    grcov

    # needed to link programs for the host
    gcc # `cc`

    # needed to execute AArch64/Arm programs
    qemu # `qemu-system-aarch64`

    # "You've met with a terrible fate, haven't you?"
    # Optional (I hope you won't need this)
    gdb
  ];
}
