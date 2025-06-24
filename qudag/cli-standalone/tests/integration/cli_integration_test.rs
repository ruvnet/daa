use std::process::Command;
use std::path::Path;
use std::fs;

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qudag", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("QuDAG node operation and management CLI"));
    assert!(stdout.contains("USAGE:"));
    assert!(stdout.contains("COMMANDS:"));
}

#[test]
fn test_start_command_help() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qudag", "--", "start", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Start the QuDAG node"));
    assert!(stdout.contains("--port"));
    assert!(stdout.contains("--data-dir"));
    assert!(stdout.contains("--peers"));
}

#[test]
fn test_peer_command_help() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qudag", "--", "peer", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Peer management commands"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("add"));
    assert!(stdout.contains("remove"));
}

#[test]
fn test_network_command_help() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qudag", "--", "network", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Network management commands"));
    assert!(stdout.contains("stats"));
    assert!(stdout.contains("test"));
}

#[test]
fn test_dag_command_help() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qudag", "--", "dag", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DAG visualization"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("--format"));
}

#[test]
fn test_status_command() {
    // This test verifies the status command runs without errors
    // In a real implementation, this would connect to a running node
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qudag", "--", "status"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check for placeholder output since no actual node is running
    assert!(stdout.contains("Node Status:") || stdout.contains("No running node"));
}

#[test]
fn test_peer_list_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qudag", "--", "peer", "list"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Connected Peers:") || stdout.contains("No peers connected"));
}

#[test]
fn test_network_stats_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qudag", "--", "network", "stats"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Network Statistics:"));
}

#[test]
fn test_network_test_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qudag", "--", "network", "test"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Network Connectivity Test Results:"));
}

#[test]
fn test_dag_visualization() {
    // Test DAG visualization with temporary output file
    let temp_file = "test_dag.dot";
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qudag", "--", "dag", "--output", temp_file])
        .output()
        .expect("Failed to execute command");

    // Check if the file was created
    assert!(Path::new(temp_file).exists());
    
    // Check file contents
    let contents = fs::read_to_string(temp_file).expect("Failed to read file");
    assert!(contents.contains("digraph DAG"));
    assert!(contents.contains("->"));
    
    // Clean up
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_invalid_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qudag", "--", "invalid-command"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error:") || stderr.contains("unrecognized subcommand"));
}