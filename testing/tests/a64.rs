use std::io;

#[test]
fn a64() -> io::Result<()> {
    testing::detect_and_run_tests("a64/packages")
}
