use std::path::PathBuf;
use std::{fs, io};

pub fn run(relpath: &str) -> io::Result<()> {
    for res in fs::read_dir(repo_root().join(relpath))? {
        let entry = res?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        eprintln!("\n# run({})", path.file_name().unwrap().to_string_lossy());
        running_wheel::run(&path)?;
    }
    Ok(())
}

pub fn repo_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path
}
