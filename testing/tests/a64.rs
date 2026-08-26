use std::io;

#[test]
fn a64() -> io::Result<()> {
    testing::run("a64/packages")
}
