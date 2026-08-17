use color_eyre::eyre::Result;
use pretty_assertions::assert_eq;
use std::fs;
use tempfile::tempdir;

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::prelude::*;
use walkdir::WalkDir;

use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn test_skip_write_does_not_write() -> Result<()> {
    let tempdir = tempdir()?;

    let mut cmd = cargo_bin_cmd!();
    cmd.arg("-p")
        .arg("tests/test_pkg")
        .arg("-s")
        .arg(tempdir.path())
        .arg("-e")
        .arg("test_pkg/miss_spelled_ref.py")
        .arg("-e")
        .arg("test_pkg/excluded_file.py")
        .arg("--exclude")
        .arg("test_pkg/excluded_module")
        .arg("--skip-write")
        .arg("-vvv");
    let assertion = cmd.assert();

    assertion.success();

    let number_of_files = std::fs::read_dir(tempdir.path())?.count();

    assert_eq!(number_of_files, 0);

    Ok(())
}
#[test]
fn test_cli_with_all_options() -> Result<()> {
    let tempdir = tempdir()?;

    let mut cmd = cargo_bin_cmd!();
    cmd.arg("-p")
        .arg("tests/test_pkg")
        .arg("-s")
        .arg(tempdir.path())
        .arg("--skip-undoc")
        .arg("--skip-private")
        .arg("-e")
        .arg("test_pkg/miss_spelled_ref.py")
        .arg("-e")
        .arg("test_pkg/excluded_file.py")
        .arg("--exclude")
        .arg("test_pkg/excluded_module")
        .arg("-vv");
    let assertion = cmd.assert();

    assertion.success();

    Ok(())
}
#[test]
fn test_cli_with_zola() -> Result<()> {
    let tempdir = tempdir()?;

    let tmp_dir_path = tempdir.path();
    let target_dir = tmp_dir_path.join("zola_test_site");

    let origin = PathBuf::from("tests/zola_test_site/");

    copy_dir_recursive(&origin, tmp_dir_path)?;

    let mut cmd = cargo_bin_cmd!();
    cmd.arg("-p")
        .arg("tests/test_pkg")
        .arg("-s")
        .arg(&target_dir)
        .arg("--skip-undoc")
        .arg("--skip-private")
        .arg("-e")
        .arg("test_pkg/miss_spelled_ref.py")
        .arg("-e")
        .arg("test_pkg/excluded_file.py")
        .arg("--exclude")
        .arg("test_pkg/excluded_module")
        .arg("--ssg")
        .arg("zola")
        .arg("-vv");
    let snakedown_assertion = cmd.assert();

    snakedown_assertion.success();

    let zola_cmd_assert = Command::new("zola")
        .current_dir(&target_dir)
        .arg("build")
        .assert();

    zola_cmd_assert.success();

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let rel_path = entry.path().strip_prefix(src)?;
        let target_path = dst.join(rel_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_path)?;
        } else {
            fs::copy(entry.path(), &target_path)?;
        }
    }

    Ok(())
}
