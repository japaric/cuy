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
  cargo +{{nightly-fmt}} fmt --all {{ARGS}}
  cd testing && cargo +{{nightly-fmt}} fmt --all {{ARGS}}

# runs all test suites
[working-directory: 'testing']
test:
  cargo t --target host-tuple -- --nocapture

clippy:
  #!/usr/bin/env bash
  set -euo pipefail
  for pkg in `ls packages/`; do
      [ -f packages/$pkg/.env ] || continue
      pushd packages/$pkg
      target=$(grep TARGET .env | cut -d '"' -f2)
      cargo clippy --examples --target $target -- -D clippy::undocumented_unsafe_blocks -D warnings -D missing_docs
      popd
  done

# runs `llvm-nm` on the disasm application
nm TARGET *ARGS:
  just build-disasm {{TARGET}}
  {{llvmdir}}/llvm-nm {{ARGS}} target/{{TARGET}}/release/disasm

# runs `llvm-objdump` on the disasm application
objdump TARGET *ARGS:
  just build-disasm {{TARGET}}
  {{llvmdir}}/llvm-objdump {{ARGS}} target/{{TARGET}}/release/disasm

# runs `llvm-size` on the disasm application
size TARGET *ARGS:
  just build-disasm {{TARGET}}
  {{llvmdir}}/llvm-size {{ARGS}} target/{{TARGET}}/release/disasm

[private]
build-disasm TARGET:
  cargo b -p disasm --target {{TARGET}} --release
