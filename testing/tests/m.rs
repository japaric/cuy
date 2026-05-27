use std::io;

#[test]
fn m() -> io::Result<()> {
    testing::detect_and_run_tests("m/packages")
}
