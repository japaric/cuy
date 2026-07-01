use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::{fs, io};

pub fn detect_and_run_tests(relpath: impl AsRef<Path>) -> io::Result<()> {
    let relpath = relpath.as_ref();

    let packages_dir = root().join(relpath);
    for entry in fs::read_dir(&packages_dir)? {
        let entry = entry?;

        if !entry.file_type()?.is_dir() {
            continue;
        }

        let pkg_path = &entry.path();
        let env_map = load_env_map(pkg_path);
        let examples_dir = pkg_path.join("examples");
        if !examples_dir.exists() {
            continue;
        }

        let pkg_name = &path_stem(pkg_path);
        for entry in fs::read_dir(&examples_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
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
            let Directives { runner, target } = &extract_directives(path)?;
            let runner = shellexpand::env_with_context_no_errors(runner, |key| env_map.get(key));
            let target = shellexpand::env_with_context_no_errors(target, |key| env_map.get(key));

            let runner_config = format!("target.'cfg(true)'.runner = '{runner}'",);
            for release in [false, true] {
                let mut cargo = Command::new("cargo");
                cargo.arg("--config").arg(&runner_config);
                cargo.arg("run");
                if release {
                    cargo.arg("--release");
                }
                if cfg!(feature = "codecov") {
                    if release {
                        continue;
                    }

                    cargo.env("RUSTFLAGS", "-Cinstrument-coverage -Zno-profiler-runtime");
                    cargo.args(["--features", "codecov"]);
                }
                cargo.args([
                    "-q",
                    "-p",
                    pkg_name,
                    "--example",
                    example_name,
                    "--target",
                    &*target,
                ]);
                eprintln!("$ {cargo:?}");
                let output = cargo
                    .current_dir(pkg_path)
                    .output()
                    .expect("`cargo run` failed");

                assert!(
                    output.status.success(),
                    "Cargo stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                );

                let stderr = String::from_utf8_lossy(&output.stderr);
                let stderr = stderr.trim();
                if !stderr.is_empty() {
                    eprintln!("{stderr}");
                }

                let actual =
                    String::from_utf8(output.stdout).expect("example's stdout is not UTF-8");
                pretty_assertions::assert_eq!(expected, actual);
            }
        }
    }

    Ok(())
}

fn extract_directives(path: &Path) -> io::Result<Directives> {
    let contents = fs::read_to_string(path)?;

    let mut runner = None;
    let mut target = None;
    for line in contents.lines() {
        let line = line.trim();
        let Some(comment) = line.strip_prefix("//@") else {
            continue;
        };

        let comment = comment.trim();
        if let Some(value) = comment.strip_prefix("runner:") {
            assert!(runner.is_none(), "runner specified more than once");
            runner = Some(value.trim().to_string());
        } else if let Some(value) = comment.strip_prefix("target:") {
            assert!(target.is_none(), "target specified more than once");
            target = Some(value.trim().to_string());
        }
    }

    let runner = runner.expect("runner was not specified");
    let target = target.expect("runner was not specified");
    Ok(Directives { runner, target })
}

struct Directives {
    runner: String,
    target: String,
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

fn load_env_map(pkg_path: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();

    if let Ok(env_iter) = dotenvy::from_filename_iter(pkg_path.join(".env")) {
        for res in env_iter {
            if let Ok((k, v)) = res {
                if k.starts_with('_') {
                    continue;
                }

                map.insert(k, v);
            }
        }
    }
    map
}
