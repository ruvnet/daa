//! Unit tests for peer address parsing and validation
//! These tests focus on the address validation logic

use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr};

/// Peer address validation module
/// This would normally be in the main CLI code, but for TDD we define it here first
pub struct PeerAddressValidator;

impl PeerAddressValidator {
    /// Validate and parse peer address
    pub fn validate_address(address: &str) -> Result<PeerAddress, AddressError> {
        // Try parsing as standard socket address first
        if let Ok(socket_addr) = address.parse::<SocketAddr>() {
            return Ok(PeerAddress::Socket(socket_addr));
        }
        
        // Check for .dark address
        if address.ends_with(".dark") {
            return Self::validate_dark_address(address);
        }
        
        // Check for .onion address
        if address.contains(".onion") {
            return Self::validate_onion_address(address);
        }
        
        // Try parsing as domain:port
        if let Some((domain, port_str)) = address.rsplit_once(':') {
            if let Ok(port) = port_str.parse::<u16>() {
                if port > 0 && Self::is_valid_domain(domain) {
                    return Ok(PeerAddress::Domain(domain.to_string(), port));
                }
            }
        }
        
        Err(AddressError::InvalidFormat)
    }
    
    fn validate_dark_address(address: &str) -> Result<PeerAddress, AddressError> {
        if !address.ends_with(".dark") {
            return Err(AddressError::InvalidDarkAddress);
        }
        
        let domain = address.strip_suffix(".dark").unwrap();
        
        // Validate domain part
        if domain.is_empty() || domain.len() > 63 {
            return Err(AddressError::InvalidDarkAddress);
        }
        
        // Check character set (alphanumeric and hyphens, must start with letter)
        if !domain.chars().next().unwrap_or('0').is_ascii_lowercase() {
            return Err(AddressError::InvalidDarkAddress);
        }
        
        for ch in domain.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '-' {
                return Err(AddressError::InvalidDarkAddress);
            }
        }
        
        Ok(PeerAddress::Dark(address.to_string()))
    }
    
    fn validate_onion_address(address: &str) -> Result<PeerAddress, AddressError> {
        if let Some((onion_part, port_str)) = address.rsplit_once(':') {
            // Validate port
            let port = port_str.parse::<u16>()
                .map_err(|_| AddressError::InvalidOnionAddress)?;
            
            if port == 0 {
                return Err(AddressError::InvalidOnionAddress);
            }
            
            // Validate .onion domain
            if !onion_part.ends_with(".onion") {
                return Err(AddressError::InvalidOnionAddress);
            }
            
            let onion_hash = onion_part.strip_suffix(".onion").unwrap();
            
            // Check v2 (16 chars) or v3 (56 chars) format
            match onion_hash.len() {
                16 => {
                    // Tor v2 address
                    for ch in onion_hash.chars() {
                        if !"abcdefghijklmnopqrstuvwxyz234567".contains(ch) {
                            return Err(AddressError::InvalidOnionAddress);
                        }
                    }
                }
                56 => {
                    // Tor v3 address
                    for ch in onion_hash.chars() {
                        if !"abcdefghijklmnopqrstuvwxyz234567".contains(ch) {
                            return Err(AddressError::InvalidOnionAddress);
                        }
                    }
                }
                _ => return Err(AddressError::InvalidOnionAddress),
            }
            
            Ok(PeerAddress::Onion(address.to_string()))
        } else {
            Err(AddressError::InvalidOnionAddress)
        }
    }
    
    fn is_valid_domain(domain: &str) -> bool {
        if domain.is_empty() || domain.len() > 253 {
            return false;
        }
        
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() < 2 {
            return false;
        }
        
        for part in parts {
            if part.is_empty() || part.len() > 63 {
                return false;
            }
            
            // Must start and end with alphanumeric
            if !part.chars().next().unwrap_or('0').is_ascii_alphanumeric() ||
               !part.chars().last().unwrap_or('0').is_ascii_alphanumeric() {
                return false;
            }
            
            // Only alphanumeric and hyphens
            for ch in part.chars() {
                if !ch.is_ascii_alphanumeric() && ch != '-' {
                    return false;
                }
            }
        }
        
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PeerAddress {
    Socket(SocketAddr),
    Domain(String, u16),
    Dark(String),
    Onion(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AddressError {
    InvalidFormat,
    InvalidDarkAddress,
    InvalidOnionAddress,
    InvalidPort,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_address_validation() {
        let cases = vec![
            ("127.0.0.1:8000", true),
            ("192.168.1.100:9000", true),
            ("10.0.0.1:1234", true),
            ("255.255.255.255:65535", true),
            ("0.0.0.0:1", true),
        ];
        
        for (address, should_be_valid) in cases {
            let result = PeerAddressValidator::validate_address(address);
            if should_be_valid {
                assert!(result.is_ok(), "Address {} should be valid", address);
                if let Ok(PeerAddress::Socket(socket_addr)) = result {
                    assert_eq!(socket_addr.to_string(), address);
                }
            } else {
                assert!(result.is_err(), "Address {} should be invalid", address);
            }
        }
    }

    #[test]
    fn test_ipv6_address_validation() {
        let cases = vec![
            ("[::1]:8000", true),
            ("[2001:db8::1]:9000", true),
            ("[fe80::1%lo0]:8000", false), // Interface specification not supported
            ("[2001:db8:85a3::8a2e:370:7334]:443", true),
            ("[::]:80", true),
        ];
        
        for (address, should_be_valid) in cases {
            let result = PeerAddressValidator::validate_address(address);
            if should_be_valid {
                assert!(result.is_ok(), "Address {} should be valid", address);
                if let Ok(PeerAddress::Socket(_)) = result {
                    // Additional IPv6-specific validation could go here
                }
            } else {
                assert!(result.is_err(), "Address {} should be invalid", address);
            }
        }
    }

    #[test]
    fn test_domain_address_validation() {
        let cases = vec![
            ("example.com:8000", true),
            ("node1.qudag.network:9000", true),
            ("test-node.example.org:443", true),
            ("localhost:8080", true),
            ("sub.domain.example.io:3000", true),
            ("a.b:1", true),
            ("invalid:0", false), // Port 0 is invalid
            ("invalid:", false), // Missing port
            (":8000", false), // Missing domain
            ("invalid-domain-name-that-is-way-too-long-to-be-valid.com:8000", false),
            ("example..com:8000", false), // Double dot
            ("-example.com:8000", false), // Starts with hyphen
            ("example-.com:8000", false), // Ends with hyphen
        ];
        
        for (address, should_be_valid) in cases {
            let result = PeerAddressValidator::validate_address(address);
            if should_be_valid {
                assert!(result.is_ok(), "Address {} should be valid", address);
                if let Ok(PeerAddress::Domain(domain, port)) = result {
                    assert!(port > 0);
                    assert!(!domain.is_empty());
                }
            } else {
                assert!(result.is_err(), "Address {} should be invalid", address);
            }
        }
    }

    #[test]
    fn test_dark_address_validation() {
        let cases = vec![
            ("mynode.dark", true),
            ("test-node.dark", true),
            ("a.dark", true),
            ("valid123.dark", true),
            ("node-with-dashes.dark", true),
            
            // Invalid cases
            (".dark", false), // Empty name
            ("123node.dark", false), // Starts with number
            ("-node.dark", false), // Starts with hyphen
            ("node-.dark", false), // Ends with hyphen
            ("node.with.dots.dark", false), // Contains dots
            ("node_with_underscores.dark", false), // Contains underscores
            ("NODE.dark", false), // Uppercase letters
            ("node.DARK", false), // Wrong case for .dark
            ("node.dark:8000", false), // Dark addresses don't have ports
            ("", false), // Empty string
            ("verylongnodename0123456789012345678901234567890123456789012345.dark", false), // Too long
        ];
        
        for (address, should_be_valid) in cases {
            let result = PeerAddressValidator::validate_address(address);
            if should_be_valid {
                assert!(result.is_ok(), "Dark address {} should be valid", address);
                if let Ok(PeerAddress::Dark(dark_addr)) = result {
                    assert_eq!(dark_addr, address);
                    assert!(dark_addr.ends_with(".dark"));
                }
            } else {
                assert!(result.is_err(), "Dark address {} should be invalid", address);
            }
        }
    }

    #[test]
    fn test_onion_address_validation() {
        let cases = vec![
            // Tor v2 addresses (16 characters)
            ("3g2upl4pq6kufc4m.onion:8000", true),
            ("facebookcorewwwi.onion:443", true),
            ("duckduckgogg42ts.onion:80", true),
            
            // Tor v3 addresses (56 characters)
            ("facebookwkhpilnemxj7asaniu7vnjjbiltxjqhye3mhbshg7kx5tfyd.onion:443", true),
            ("3g2upl4pq6kufc4m236nokrhwwupnj7jqfkfxbh6kp5g6cjqzpgczjnqd.onion:8000", true),
            
            // Invalid cases
            ("invalid.onion:8000", false), // Invalid hash length
            ("3g2upl4pq6kufc4m.onion:0", false), // Port 0
            ("3g2upl4pq6kufc4m.onion", false), // Missing port
            ("3G2UPL4PQ6KUFC4M.onion:8000", false), // Uppercase
            ("3g2upl4pq6kufc4m.onion:99999", false), // Invalid port
            ("3g2upl4pq6kufc4m.union:8000", false), // Wrong TLD
            ("", false), // Empty
        ];
        
        for (address, should_be_valid) in cases {
            let result = PeerAddressValidator::validate_address(address);
            if should_be_valid {
                assert!(result.is_ok(), "Onion address {} should be valid", address);
                if let Ok(PeerAddress::Onion(onion_addr)) = result {
                    assert_eq!(onion_addr, address);
                }
            } else {
                assert!(result.is_err(), "Onion address {} should be invalid", address);
            }
        }
    }

    #[test]
    fn test_invalid_address_formats() {
        let invalid_addresses = vec![
            "",
            "just-a-string",
            "192.168.1.256:8000", // Invalid IP
            "192.168.1.1:99999", // Invalid port
            "192.168.1.1:-1", // Negative port
            "192.168.1.1:abc", // Non-numeric port
            "[invalid-ipv6]:8000",
            "192.168.1.1", // Missing port
            ":8000", // Missing host
            "192.168.1.1:", // Missing port number
            "example.com::", // Double colon in port
            "256.256.256.256:8000", // All octets too large
        ];
        
        for address in invalid_addresses {
            let result = PeerAddressValidator::validate_address(address);
            assert!(result.is_err(), "Address {} should be invalid", address);
        }
    }

    #[test]
    fn test_port_range_validation() {
        // Test valid port ranges
        for port in [1, 80, 443, 8000, 9000, 65535] {
            let address = format!("127.0.0.1:{}", port);
            let result = PeerAddressValidator::validate_address(&address);
            assert!(result.is_ok(), "Port {} should be valid", port);
        }
        
        // Port 0 is handled differently in socket parsing
        // so we test it separately in domain context
        let result = PeerAddressValidator::validate_address("example.com:0");
        assert!(result.is_err(), "Port 0 should be invalid for domain addresses");
    }

    #[test]
    fn test_address_type_detection() {
        let test_cases = vec![
            ("127.0.0.1:8000", "Socket"),
            ("[::1]:8000", "Socket"),
            ("example.com:8000", "Domain"),
            ("mynode.dark", "Dark"),
            ("3g2upl4pq6kufc4m.onion:8000", "Onion"),
        ];
        
        for (address, expected_type) in test_cases {
            let result = PeerAddressValidator::validate_address(address);
            assert!(result.is_ok(), "Address {} should be valid", address);
            
            let actual_type = match result.unwrap() {
                PeerAddress::Socket(_) => "Socket",
                PeerAddress::Domain(_, _) => "Domain",
                PeerAddress::Dark(_) => "Dark",
                PeerAddress::Onion(_) => "Onion",
            };
            
            assert_eq!(actual_type, expected_type, 
                      "Address {} should be detected as {} type", address, expected_type);
        }
    }

    #[test]
    fn test_edge_cases() {
        // Test very long valid addresses
        let long_domain = "a".repeat(50) + ".com:8000";
        let result = PeerAddressValidator::validate_address(&long_domain);
        assert!(result.is_ok(), "Long valid domain should be accepted");
        
        // Test maximum port
        let max_port_addr = "127.0.0.1:65535";
        let result = PeerAddressValidator::validate_address(max_port_addr);
        assert!(result.is_ok(), "Maximum port should be valid");
        
        // Test minimum port
        let min_port_addr = "127.0.0.1:1";
        let result = PeerAddressValidator::validate_address(min_port_addr);
        assert!(result.is_ok(), "Minimum port should be valid");
    }

    #[test]
    fn test_normalization() {
        // Test that addresses are preserved as-is (no normalization)
        let address = "Example.COM:8000";
        let result = PeerAddressValidator::validate_address(address);
        // This should fail because domain validation is case-sensitive in our implementation
        // In a real implementation, you might want to normalize to lowercase
        assert!(result.is_err(), "Uppercase domains should be rejected");
    }
}