use std::{path::PathBuf, process::Command};

use insta::{assert_snapshot, with_settings, Settings};
use tempdir::TempDir;

pub(crate) fn test_main(path: &str, name: &str) {
    let mut cmd = Command::new(env!("CARGO"));

    cmd.arg("run");
    cmd.arg("--example");
    cmd.arg(name);

    cmd.env("RUSTFLAGS", "--cap-lints=allow");
    cmd.env("CARGO_TERM_COLOR", "never");

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

    let filter_locking = r".*Locking \d+ packages to latest compatible versions\n";
    settings.add_filter(filter_locking, "");

    let filter_adding = r".*Adding .* (\(.*\))?\n";
    settings.add_filter(filter_adding, "");

    let filter_path = if cfg!(windows) {
        r"(?:([\w\-_]+)(?:\\))+([\w\-_]+?\.[\w\-_]+)?"
    } else {
        r"(?:([\w\-_]+)(?:/))+([\w\-_]+?\.[\w\-_]+)?"
    };
    settings.add_filter(filter_path, "$1/$2");

    if cfg!(windows) {
        // `\\` has already been replaced by `/`
        let filter_exe = r"Running `(?:([\w\-_]+)(?:/))+([\w\-_]+?).exe`";
        settings.add_filter(filter_exe, "Running `$1/$2`");
    }

    let pretty = format!(
        "Stdout:\n{}\n\nStderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    settings.bind(|| {
        with_settings!({snapshot_suffix => name}, {
            assert_snapshot!(pretty);
        });
    });
}
