alias t := test

# last nightly version that passed minicov's CI
nightly-minicov := 'nightly-2025-12-04'
nightly-fmt := 'nightly-2026-02-24'
host := `rustc --print host-tuple`
sysroot := `rustc --print sysroot`
target := `grep 'target =' .cargo/config.toml | cut -d'"' -f2`

llvmdir := sysroot + '/lib/rustlib/' + host + '/bin'
releasedir := 'target/' + target + '/release/'
disasmpath := releasedir + 'disasm'

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

codecov:
  rm -f *.profraw
  rustup toolchain install {{nightly-minicov}} --profile minimal --target {{target}}
  cd testing && cargo +{{nightly-minicov}} t --target host-tuple --features codecov -- --nocapture
  grcov . -s . --binary-path ./target/{{target}}/debug -t html --branch --ignore-not-existing -o ./target/coverage/
  rm -f *.profraw

clippy:
  cargo clippy --examples --workspace -- -D clippy::undocumented_unsafe_blocks -D warnings -D missing_docs

# runs `llvm-nm` on the disasm application
nm *ARGS: build-disasm
  {{llvmdir}}/llvm-nm {{ARGS}} {{disasmpath}}

# runs `llvm-objdump` on the disasm application
objdump *ARGS: build-disasm
  {{llvmdir}}/llvm-objdump {{ARGS}} {{disasmpath}}

# runs `llvm-size` on the disasm application
size *ARGS: build-disasm
  {{llvmdir}}/llvm-size {{ARGS}} {{disasmpath}}

[private]
build-disasm:
  cargo b -p disasm --release
