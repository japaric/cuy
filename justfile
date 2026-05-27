alias t := test

# last nightly version that passed minicov's CI
nightly-minicov := 'nightly-2025-12-04'
nightly-fmt := 'nightly-2026-02-24'
host := `rustc --print host-tuple`
sysroot := `rustc --print sysroot`

llvmdir := sysroot + '/lib/rustlib/' + host + '/bin'

pre-commit-check:
  git diff --quiet || exit 1
  just test
  just clippy
  just fmt --check

setup:
   cp pre-commit.bash .git/hooks/pre-commit

fmt *ARGS:
  rustup toolchain install {{nightly-fmt}} --profile minimal --component rustfmt
  cd a64 && cargo +{{nightly-fmt}} fmt --all {{ARGS}}
  cd m && cargo +{{nightly-fmt}} fmt --all {{ARGS}}
  cd testing && cargo +{{nightly-fmt}} fmt --all {{ARGS}}

# runs all test suites
test:
  cd testing && cargo t --target host-tuple -- --nocapture
  cd any/packages/whoarchi && cargo t --target host-tuple

clippy:
  #!/usr/bin/env bash
  set -euo pipefail
  profiles="a64 m"
  for profile in ${profiles}; do
    for pkg in `ls $profile/packages/`; do
        [ -f $profile/packages/$pkg/.env ] || continue
        pushd $profile/packages/$pkg
        target=$(grep TARGET .env | cut -d '"' -f2)
        cargo clippy --examples --target $target -- -D clippy::undocumented_unsafe_blocks -D warnings -D missing_docs
        popd
    done
  done
  cd any/packages/whoarchi && cargo clippy --target host-tuple -- -D clippy::undocumented_unsafe_blocks -D warnings -D missing_docs

# runs `llvm-nm` on the disasm application
[working-directory: 'any/packages/disasm']
nm TARGET *ARGS:
  just build-disasm {{TARGET}}
  {{llvmdir}}/llvm-nm {{ARGS}} target/{{TARGET}}/release/disasm

# runs `llvm-objdump` on the disasm application
[working-directory: 'any/packages/disasm']
objdump TARGET *ARGS:
  just build-disasm {{TARGET}}
  {{llvmdir}}/llvm-objdump {{ARGS}} target/{{TARGET}}/release/disasm

# runs `llvm-size` on the disasm application
[working-directory: 'any/packages/disasm']
size TARGET *ARGS:
  just build-disasm {{TARGET}}
  {{llvmdir}}/llvm-size {{ARGS}} target/{{TARGET}}/release/disasm


[private]
[working-directory: 'any/packages/disasm']
build-disasm TARGET:
  cargo b -p disasm --target {{TARGET}} --release
