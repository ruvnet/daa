use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("prime").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Prime distributed ML CLI"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("prime").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("prime"));
}

#[test]
fn test_node_start_command() {
    let mut cmd = Command::cargo_bin("prime").unwrap();
    cmd.arg("node")
        .arg("start")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Start a Prime node"));
}

#[test]
fn test_train_command() {
    let mut cmd = Command::cargo_bin("prime").unwrap();
    cmd.arg("train")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Start distributed training"));
}

#[test]
fn test_config_init() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    
    let mut cmd = Command::cargo_bin("prime").unwrap();
    cmd.arg("config")
        .arg("init")
        .arg("--path")
        .arg(config_path.to_str().unwrap())
        .assert()
        .success();
    
    assert!(config_path.exists());
}

#[test]
fn test_bootstrap_node() {
    let mut cmd = Command::cargo_bin("prime").unwrap();
    cmd.arg("bootstrap")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bootstrap a new Prime network"));
}