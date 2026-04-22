# `testing`

> Test discovery and execution

## Operation

All binary crates in the `/packages/*/examples` directories are treated as test programs.

All test programs are expected to exit with code 0.

The stdout of each test program is checked against the contents of its accompanying `.stdout` file, if there is one.
An accompanying 
If a `.stdout` file is absent then the test is expected to produce no output.

The stderr of each test program is not checked and its content does *not* have to be reproducible.

## Directives

Each test program encodes metadata in the form of directives.
Directives are single-line comments of the form:

``` rust
// key: value goes here
```

The supported directives are document in the following sub-sections.
The name of the sub-section matches the `key` component of the directive.

### `runner`

This is a mandatory directive that specfies the runner that Cargo will use to run the test program.
