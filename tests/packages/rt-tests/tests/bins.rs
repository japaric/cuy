use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

type TestResult<T> = Result<T, Box<dyn Error>>;

const PKG_NAME: &str = "rt-tests";

// TODO do test discovery instead of maintaining a manual list if `insta` cooperates

/// REQ000
#[test]
fn bss_is_zeroed() -> TestResult<()> {
    let stdout = cargo_run("bss-is-zeroed")?;
    insta::assert_snapshot!(stdout);
    Ok(())
}

#[test]
fn data_is_initialized() -> TestResult<()> {
    let stdout = cargo_run("data-is-initialized")?;
    insta::assert_snapshot!(stdout);
    Ok(())
}

#[test]
fn boot_el1() -> TestResult<()> {
    let stdout = cargo_run("boot-el1")?;
    insta::assert_snapshot!(stdout);
    Ok(())
}

#[test]
fn boot_el2() -> TestResult<()> {
    let stdout = cargo_run("boot-el2")?;
    insta::assert_snapshot!(stdout);
    Ok(())
}

#[test]
fn boot_el3() -> TestResult<()> {
    let stdout = cargo_run("boot-el3")?;
    insta::assert_snapshot!(stdout);
    Ok(())
}

#[test]
fn el2_to_el1() -> TestResult<()> {
    let stdout = cargo_run("el2-to-el1")?;
    insta::assert_snapshot!(stdout);
    Ok(())
}

fn cargo_run(name: &str) -> TestResult<String> {
    let target = target();
    let directives = parse_directives(name);

    let mut cargo = Command::new("cargo");

    cargo.args(["run", "--target", &target, "-p", PKG_NAME, "--bin", name]);
    if cfg!(feature = "codecov") {
        cargo.env("RUSTFLAGS", "-Cinstrument-coverage -Zno-profiler-runtime");
        cargo.args(["--features", "codecov"]);
    }

    let output = cargo
        .env(runner_key(), runner_value(&directives))
        .current_dir(repo_root())
        .output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Cargo command failed with: {stderr}"
    );
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout)
}

fn runner_key() -> String {
    let target = target().replace("-", "_").to_ascii_uppercase();
    format!("CARGO_TARGET_{target}_RUNNER")
}

fn runner_value(directives: &Directives) -> String {
    let machine_option = match directives.boot {
        Boot::EL1 => "",
        Boot::EL2 => ",virtualization=on",
        Boot::EL3 => ",secure=on",
    };

    // TODO probably want to test different CPUs
    format!(
        "qemu-system-aarch64 \
         -cpu neoverse-v1 \
         -machine virt{machine_option} \
         -nographic \
         -semihosting \
         -smp 2 \
         -kernel"
    )
}

fn parse_directives(name: &str) -> Directives {
    let path = repo_root()
        .join("packages")
        .join(PKG_NAME)
        .join("src")
        .join("bin")
        .join(name)
        .with_extension("rs");
    let contents = fs::read_to_string(path).expect("did not find source for {name}");
    let mut boot = None;
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("//@") {
            let rest = rest.trim();

            if let Some(rest) = rest.strip_prefix("boot-") {
                boot = Some(match rest {
                    "el1" => Boot::EL1,
                    "el2" => Boot::EL2,
                    "el3" => Boot::EL3,
                    _ => panic!("unknown directive: {rest}"),
                })
            } else {
                panic!("unknown directive: {rest}")
            }
        }
    }

    Directives {
        boot: boot.unwrap_or_default(),
    }
}

struct Directives {
    boot: Boot,
}

#[derive(Default)]
enum Boot {
    #[default]
    EL1,
    EL2,
    EL3,
}

fn repo_root() -> &'static Path {
    manifest_dir()
        .ancestors()
        .nth(3)
        .expect("unexpected repository layout")
}

fn target() -> String {
    let path = repo_root().join(".cargo").join("config.toml");
    let toml = fs::read_to_string(path).expect("could not read .cargo/config.toml");
    for line in toml.lines() {
        if let Some(rest) = line.strip_prefix("target = ") {
            return rest.trim_matches(|c| c == '"').to_string();
        }
    }

    panic!("did not find `build.target` entry in `.cargo/config.toml`")
}

fn manifest_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
