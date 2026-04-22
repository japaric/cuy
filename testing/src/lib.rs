use std::path::Path;
use std::process::Command;
use std::{fs, io};

const TARGET: &str = "aarch64-unknown-none";

#[test]
fn snapshot_tests() -> io::Result<()> {
    let packages_dir = root().join("packages");
    for entry in fs::read_dir(&packages_dir)? {
        let entry = entry?;

        if !entry.file_type()?.is_dir() {
            continue;
        }

        let pkg_path = &entry.path();
        let examples_dir = pkg_path.join("examples");
        if !examples_dir.exists() {
            continue;
        }

        let pkg_name = &path_stem(pkg_path);
        for entry in fs::read_dir(&examples_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }

            let path = &entry.path();
            let extension = path.extension().unwrap_or_default();
            if extension != "rs" {
                continue;
            }

            let example_name = &path_stem(path);
            eprintln!("\nrun packages/{pkg_name}/examples/{example_name}");
            let expected = fs::read_to_string(path.with_extension("stdout")).unwrap_or_default();
            let Directives { runner } = &extract_directives(path)?;

            let runner_key = format!(
                "CARGO_TARGET_{}_RUNNER",
                TARGET.replace("-", "_").to_ascii_uppercase()
            );
            let mut cargo = Command::new("cargo");
            cargo.arg("run");
            if cfg!(feature = "codecov") {
                cargo.env("RUSTFLAGS", "-Cinstrument-coverage -Zno-profiler-runtime");
                cargo.args(["--features", "codecov"]);
            }
            cargo.args([
                "-p",
                &pkg_name,
                "--example",
                example_name,
                "--target",
                TARGET,
            ]);
            eprintln!("$ {cargo:?}");
            let output = cargo
                .env(runner_key, &runner)
                .current_dir(root())
                .output()
                .expect("`cargo run` failed");

            assert!(
                output.status.success(),
                "Cargo stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );

            let actual = String::from_utf8(output.stdout).expect("example's stdout is not UTF-8");
            pretty_assertions::assert_eq!(expected, actual);
        }
    }

    Ok(())
}

fn extract_directives(path: &Path) -> io::Result<Directives> {
    let contents = fs::read_to_string(path)?;

    let mut runner = None;
    for line in contents.lines() {
        let line = line.trim();
        let Some(comment) = line.strip_prefix("//") else {
            continue;
        };

        let comment = comment.trim();
        if let Some(value) = comment.strip_prefix("runner:") {
            assert!(runner.is_none(), "runner specified more than once");
            runner = Some(value.trim().to_string());
        }
    }

    let runner = runner.expect("runner was not specified");
    Ok(Directives { runner })
}

struct Directives {
    runner: String,
}

fn path_stem(path: &Path) -> &str {
    path.file_stem()
        .expect("path has no file stem")
        .to_str()
        .expect("file stem is not UTF-8")
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .expect("project layout changed; this needs to be updated")
}
