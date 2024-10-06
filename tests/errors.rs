use std::{path::PathBuf, process::Command};

use insta::{assert_snapshot, with_settings, Settings};
use tempdir::TempDir;
use test_case::test_case;

#[test_case("coffee", "blueprint")]
#[test_case("coffee", "missing_provides")]
#[test_case("coffee", "use_inject_on_impl")]
#[test_case("coffee", "component_on_impl")]
#[test_case("coffee", "component_type_mismatch")]
#[test_case("coffee", "component_type_mismatch_generics")]
#[test_case("coffee", "component_missing_dependency")]
#[test_case("coffee", "component_missing_binding")]
#[test_case("coffee", "provides_on_trait")]
#[test_case("coffee", "provides_on_empty_impl")]
#[test_case("coffee", "provides_on_impl_with_more_than_one_function")]
#[test_case("coffee", "provides_invalid_return_type")]
fn test_errors_coffee(path: &str, name: &str) {
    let mut cmd = Command::new(env!("CARGO"));

    cmd.arg("run");
    cmd.arg("--example");
    cmd.arg(name);

    cmd.env("RUSTFLAGS", "--cap-lints=allow");

    let mut current_dir = PathBuf::new();
    current_dir.push("test-data");
    current_dir.push(path);

    cmd.current_dir(&current_dir);

    let target_dir = TempDir::new_in(env!("CARGO_TARGET_TMPDIR"), name).unwrap();
    cmd.env("TARGET_DIR", target_dir.path());

    let output = cmd
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute test {name}"));

    let mut settings = Settings::clone_current();

    let filter_block_package_cache = r".*Blocking waiting for file lock on package cache.*\n";
    settings.add_filter(filter_block_package_cache, "");

    let filter_block_build_directory = r".*Blocking waiting for file lock on build directory.*\n";
    settings.add_filter(filter_block_build_directory, "");

    let filter_finished_targets = r".*Finished .* target\(s\) in .*s";
    settings.add_filter(filter_finished_targets, "Finished compiling target(s)");

    let filter_compiling = r".*Compiling .*\n";
    settings.add_filter(filter_compiling, "");

    let filter_updating_index = r".*Updating crates.io index\n";
    settings.add_filter(filter_updating_index, "");

    let pretty = format!(
        "Status {:?}\n\nStdout:\n{}\n\nStderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    settings.bind(|| {
        with_settings!({snapshot_suffix => name}, {
            assert_snapshot!(pretty);
        });
    });
}
