use std::{path::PathBuf, process::Command};

use insta::{assert_compact_debug_snapshot, assert_snapshot, with_settings, Settings};
use tempdir::TempDir;
use test_case::test_case;

#[test_case("blueprint")]
#[test_case("missing_inject")]
#[test_case("component_on_impl")]
#[test_case("component_type_mismatch")]
#[test_case("component_type_mismatch_generics")]
#[test_case("component_missing_dependency")]
#[test_case("component_missing_binding")]
#[test_case("inject_on_trait")]
#[test_case("inject_on_empty_impl")]
#[test_case("inject_on_impl_with_more_than_one_function")]
#[test_case("inject_invalid_return_type")]
fn test_errors(name: &str) {
    let mut cmd = Command::new(env!("CARGO"));

    cmd.arg("run");
    cmd.arg("--example");
    cmd.arg(name);

    cmd.env("RUSTFLAGS", "--cap-lints=allow");

    let mut current_dir = PathBuf::new();
    current_dir.push("test-data");
    current_dir.push("errors");

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

    settings.bind(|| {
        with_settings!({snapshot_suffix => format!("{name}@status")}, {
            assert_compact_debug_snapshot!(output.status);
        });

        with_settings!({snapshot_suffix => format!("{name}@stdout")}, {
            assert_snapshot!(String::from_utf8_lossy(&output.stdout));
        });

        with_settings!({snapshot_suffix => format!("{name}@stderr")}, {
            assert_snapshot!(String::from_utf8_lossy(&output.stderr));
        });
    });
}