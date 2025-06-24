//! Integration tests for peer management functionality
//! These tests verify the integration between CLI and peer management system

use assert_cmd::Command;
use predicates::prelude::*;
use std::process::Stdio;
use tempfile::TempDir;

/// Test basic peer command availability
#[test]
fn test_peer_commands_available() {
    // Test that peer subcommand exists
    Command::cargo_bin("qudag")
        .unwrap()
        .arg("peer")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Peer management commands"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("remove"));
}

/// Test peer list command structure
#[test]
fn test_peer_list_help() {
    Command::cargo_bin("qudag")
        .unwrap()
        .args(&["peer", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List connected peers"));
}

/// Test peer add command structure
#[test]
fn test_peer_add_help() {
    Command::cargo_bin("qudag")
        .unwrap()
        .args(&["peer", "add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Add a peer"))
        .stdout(predicate::str::contains("address"));
}

/// Test peer remove command structure
#[test]
fn test_peer_remove_help() {
    Command::cargo_bin("qudag")
        .unwrap()
        .args(&["peer", "remove", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Remove a peer"))
        .stdout(predicate::str::contains("address"));
}

/// Test that commands provide appropriate error messages
#[test]
fn test_peer_add_without_address() {
    Command::cargo_bin("qudag")
        .unwrap()
        .args(&["peer", "add"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments were not provided"));
}

/// Test environment variable support for peer operations
#[test]
fn test_peer_env_variables() {
    Command::cargo_bin("qudag")
        .unwrap()
        .env("QUDAG_MAX_PEERS", "100")
        .env("QUDAG_PEER_TIMEOUT", "30")
        .args(&["peer", "list"])
        .assert()
        .success();
}

/// Test configuration file support
#[test]
fn test_peer_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.toml");
    
    std::fs::write(&config_file, r#"
[network]
max_peers = 100
peer_timeout = 30

[peers]
bootstrap = [
    "192.168.1.100:8000",
    "192.168.1.101:8000"
]
"#).unwrap();
    
    Command::cargo_bin("qudag")
        .unwrap()
        .env("QUDAG_CONFIG", config_file.to_str().unwrap())
        .args(&["peer", "list"])
        .assert()
        .success();
}