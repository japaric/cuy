# `cuy`

CUY is not an acronym ... it's a guinea pig.

> What are you building?

An example of how to write bare metal / no-std code that includes tests that exercise MMIO / side-effect-full code and not just tests "pure" business logic.

For simplicity, this example is written such that tests can run on QEMU in system emulation mode but most tests could as well be running on real hardware ("Hardware In the Loop testing" AKA HIL). But if it were written to do HIL testing then it'd be harder for you to run the code locally so QEMU it is.

## Dependencies

See `shell.nix`

## Run tests

Run `just test`

## Run an example

Run `cargo run` but don't expect anything exciting to happen.

## Produce code coverage report

Should be

``` console
$ just codecov
```

but `minicov` appears to not working be when cross compiling.

## References

- `ARM-R64`: Arm Architecture Reference Manual for R-profile AArch64 architecture. DDI 0628.
