#![no_main]
use libfuzzer_sys::fuzz_target;
use std::str;

// Mock CLI command enum to match intended implementation
#[derive(Debug, PartialEq)]
enum CliCommand {
    Start { peer_id: Option<String>, port: Option<u16> },
    Stop,
    Status,
    Connect { address: String },
    SendMessage { target: String, message: String },
    ListPeers,
    Invalid(String),
}

// Mock CLI parser that will be implemented in tools/cli
fn parse_command(input: &str) -> Result<CliCommand, String> {
    let sanitized = sanitize_input(input);
    let parts: Vec<&str> = sanitized.trim().split_whitespace().collect();
    
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    match parts[0] {
        "start" => {
            let mut peer_id = None;
            let mut port = None;
            let mut i = 1;
            
            while i < parts.len() {
                match parts[i] {
                    "--peer-id" | "-p" => {
                        i += 1;
                        if i < parts.len() {
                            let id = parts[i].to_string();
                            if validate_peer_id(&id) {
                                peer_id = Some(id);
                            } else {
                                return Err("Invalid peer ID format".to_string());
                            }
                        } else {
                            return Err("Missing peer ID value".to_string());
                        }
                    }
                    "--port" => {
                        i += 1;
                        if i < parts.len() {
                            port = match parts[i].parse::<u16>() {
                                Ok(p) if validate_port(p) => Some(p),
                                Ok(_) => return Err("Port out of valid range".to_string()),
                                Err(_) => return Err("Invalid port number".to_string()),
                            };
                        } else {
                            return Err("Missing port value".to_string());
                        }
                    }
                    arg if arg.starts_with('-') => {
                        return Err(format!("Unknown start argument: {}", arg));
                    }
                    _ => return Err(format!("Unexpected argument: {}", parts[i])),
                }
                i += 1;
            }
            Ok(CliCommand::Start { peer_id, port })
        }
        "stop" => {
            if parts.len() > 1 {
                return Err("Stop command takes no arguments".to_string());
            }
            Ok(CliCommand::Stop)
        }
        "status" => {
            if parts.len() > 1 {
                return Err("Status command takes no arguments".to_string());
            }
            Ok(CliCommand::Status)
        }
        "connect" => {
            if parts.len() != 2 {
                return Err("Connect command requires exactly one address argument".to_string());
            }
            let address = parts[1].to_string();
            if !validate_address(&address) {
                return Err("Invalid address format".to_string());
            }
            Ok(CliCommand::Connect { address })
        }
        "send" => {
            if parts.len() < 3 {
                return Err("Send command requires target and message arguments".to_string());
            }
            let target = parts[1].to_string();
            if !validate_peer_id(&target) {
                return Err("Invalid target peer ID".to_string());
            }
            let message = parts[2..].join(" ");
            if message.len() > 1024 {
                return Err("Message too long".to_string());
            }
            Ok(CliCommand::SendMessage { target, message })
        }
        "peers" | "list-peers" => {
            if parts.len() > 1 {
                return Err("Peers command takes no arguments".to_string());
            }
            Ok(CliCommand::ListPeers)
        }
        cmd => Ok(CliCommand::Invalid(cmd.to_string())),
    }
}

/// Sanitize command input to prevent injection attacks
fn sanitize_input(input: &str) -> String {
    input.chars()
        .filter(|c| c.is_alphanumeric() || " .-_:".contains(*c))
        .take(1024) // Limit input length
        .collect()
}

/// Argument validation
fn validate_peer_id(peer_id: &str) -> bool {
    // Peer ID validation - alphanumeric, length 1-64, no special chars except underscore
    !peer_id.is_empty() 
        && peer_id.len() <= 64 
        && peer_id.chars().all(|c| c.is_alphanumeric() || c == '_')
        && !peer_id.starts_with('_')
        && !peer_id.ends_with('_')
}

fn validate_port(port: u16) -> bool {
    // Port validation - avoid reserved ports and ensure valid range
    port >= 1024 && port <= 65535
}

fn validate_address(address: &str) -> bool {
    // Address validation - support IPv4, IPv6, and hostnames
    let parts: Vec<&str> = address.split(':').collect();
    if parts.len() < 2 || parts.len() > 8 {
        return false;
    }
    
    // Extract port (last part)
    if let Some(port_str) = parts.last() {
        if let Ok(port) = port_str.parse::<u16>() {
            if !validate_port(port) {
                return false;
            }
        } else {
            return false;
        }
    }
    
    // Basic format validation for the host part
    let host = &parts[..parts.len()-1].join(":");
    !host.is_empty() && host.len() <= 253
}

/// Test command execution security
fn test_command_security(cmd: &CliCommand) -> bool {
    match cmd {
        CliCommand::Start { peer_id, port } => {
            if let Some(id) = peer_id {
                // Check for injection attempts
                if id.contains("..") || id.contains("/") || id.contains("\\") {
                    return false;
                }
            }
            if let Some(p) = port {
                // Ensure port is in safe range
                if *p < 1024 || *p > 65535 {
                    return false;
                }
            }
            true
        }
        CliCommand::Connect { address } => {
            // Check for suspicious patterns
            !address.contains("..") && !address.contains("localhost") && address.len() < 256
        }
        CliCommand::SendMessage { target, message } => {
            // Validate message content
            !message.contains('\0') && 
            !message.contains("../") && 
            message.len() <= 1024 &&
            validate_peer_id(target)
        }
        CliCommand::Invalid(cmd) => {
            // Invalid commands should be short and not contain suspicious patterns
            cmd.len() <= 32 && !cmd.contains("..") && !cmd.contains("/")
        }
        _ => true,
    }
}

// Fuzz target
fuzz_target!(|data: &[u8]| {
    // Try to convert input bytes to string
    if let Ok(s) = str::from_utf8(data) {
        // Test command parsing with various inputs
        match parse_command(s) {
            Ok(cmd) => {
                // Test command security
                assert!(test_command_security(&cmd), "Command failed security check: {:?}", cmd);
                
                // Validate parsed command arguments
                match cmd {
                    CliCommand::Start { peer_id, port } => {
                        if let Some(id) = peer_id {
                            assert!(validate_peer_id(&id), "Invalid peer ID passed validation");
                        }
                        if let Some(p) = port {
                            assert!(validate_port(p), "Invalid port passed validation");
                        }
                    }
                    CliCommand::Connect { address } => {
                        assert!(validate_address(&address), "Invalid address passed validation");
                    }
                    CliCommand::SendMessage { target, message } => {
                        assert!(validate_peer_id(&target), "Invalid target passed validation");
                        assert!(message.len() <= 1024, "Message too long");
                    }
                    _ => {}
                }
            }
            Err(error) => {
                // Error messages should not contain sensitive information
                assert!(!error.contains("password"));
                assert!(!error.contains("key"));
                assert!(!error.contains("secret"));
                assert!(error.len() <= 256, "Error message too long");
            }
        }
    }

    // Test input sanitization with various byte sequences
    if !data.is_empty() {
        let sanitized = sanitize_input(&String::from_utf8_lossy(data));
        
        // Sanitized input should be safe
        assert!(!sanitized.contains('\0'));
        assert!(!sanitized.contains('\n'));
        assert!(!sanitized.contains('\r'));
        assert!(sanitized.len() <= 1024);
        
        // Test edge cases
        let edge_cases = vec![
            "",
            " ",
            "\t\n\r",
            "a".repeat(2000),
            "../../../etc/passwd",
            "rm -rf /",
            "'; DROP TABLE users; --",
            "\x00\x01\x02\x03",
        ];
        
        for case in edge_cases {
            let sanitized_case = sanitize_input(case);
            let _ = parse_command(&sanitized_case);
        }
    }

    // Test with malformed UTF-8
    let mut malformed = data.to_vec();
    if !malformed.is_empty() {
        // Insert invalid UTF-8 sequences
        malformed[0] = 0xFF;
        if malformed.len() > 1 {
            malformed[1] = 0xFE;
        }
        
        // Should handle gracefully
        let lossy_string = String::from_utf8_lossy(&malformed);
        let _ = parse_command(&lossy_string);
    }
});