use qudag_network::dark_resolver::{DarkResolver, DarkResolverError, DarkDomainRecord};
use qudag_network::types::NetworkAddress;
use qudag_crypto::ml_kem::{MlKem768, KeyEncapsulation};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_dark_domain_registration_and_lookup() {
    let resolver = Arc::new(DarkResolver::new());
    
    // Test domain registration
    let domain = "test-service.dark";
    let address = NetworkAddress::new([192, 168, 1, 100], 8080);
    
    // Register the domain
    let result = resolver.register_domain(domain, address.clone());
    assert!(result.is_ok(), "Failed to register domain: {:?}", result);
    
    // Lookup the domain
    let record = resolver.lookup_domain(domain).unwrap();
    assert!(record.public_key.len() > 0);
    assert!(record.encrypted_address.len() > 0);
    assert!(record.registered_at > 0);
}

#[tokio::test]
async fn test_dark_domain_duplicate_registration() {
    let resolver = Arc::new(DarkResolver::new());
    
    let domain = "duplicate.dark";
    let address = NetworkAddress::new([10, 0, 0, 1], 9000);
    
    // First registration should succeed
    resolver.register_domain(domain, address.clone()).unwrap();
    
    // Second registration should fail
    let result = resolver.register_domain(domain, address);
    assert!(matches!(result, Err(DarkResolverError::DomainExists)));
}

#[tokio::test]
async fn test_dark_domain_resolution_with_decryption() {
    let resolver = Arc::new(DarkResolver::new());
    
    let domain = "encrypted-service.dark";
    let original_address = NetworkAddress::new([172, 16, 0, 50], 443);
    
    // Register domain
    resolver.register_domain(domain, original_address.clone()).unwrap();
    
    // Get the domain record
    let record = resolver.lookup_domain(domain).unwrap();
    
    // Generate a new keypair for decryption test
    let (public_key, secret_key) = MlKem768::keygen().unwrap();
    
    // Try to resolve with wrong key (should fail)
    let result = resolver.resolve_address(domain, secret_key.as_ref());
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_dark_domain_formats() {
    let resolver = Arc::new(DarkResolver::new());
    let address = NetworkAddress::new([127, 0, 0, 1], 8080);
    
    // Test various invalid domain formats
    let invalid_domains = vec![
        "no-extension",
        ".dark",
        "test.darknet",
        "test@invalid.dark",
        "test space.dark",
        "test#hash.dark",
        "",
        "a.dark", // Too short
    ];
    
    for invalid_domain in invalid_domains {
        let result = resolver.register_domain(invalid_domain, address.clone());
        assert!(
            matches!(result, Err(DarkResolverError::InvalidDomain)),
            "Domain '{}' should be invalid", 
            invalid_domain
        );
    }
}

#[tokio::test]
async fn test_concurrent_dark_domain_operations() {
    let resolver = Arc::new(DarkResolver::new());
    let mut handles = vec![];
    
    // Spawn multiple tasks to register different domains
    for i in 0..10 {
        let resolver_clone = resolver.clone();
        let handle = tokio::spawn(async move {
            let domain = format!("concurrent-{}.dark", i);
            let address = NetworkAddress::new([10, 0, 0, i as u8], 8080 + i as u16);
            resolver_clone.register_domain(&domain, address)
        });
        handles.push(handle);
    }
    
    // Wait for all registrations
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
    
    // Verify all domains are registered
    for i in 0..10 {
        let domain = format!("concurrent-{}.dark", i);
        let record = resolver.lookup_domain(&domain).unwrap();
        assert!(record.registered_at > 0);
    }
}

#[tokio::test]
async fn test_dark_domain_expiration_handling() {
    let resolver = Arc::new(DarkResolver::new());
    
    // Register a domain
    let domain = "expiring.dark";
    let address = NetworkAddress::new([192, 168, 100, 1], 8443);
    resolver.register_domain(domain, address).unwrap();
    
    // Get the record and check timestamp
    let record = resolver.lookup_domain(domain).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    assert!(record.registered_at <= now);
    assert!(record.registered_at > now - 10); // Registered within last 10 seconds
}

#[tokio::test]
async fn test_dark_domain_lookup_nonexistent() {
    let resolver = Arc::new(DarkResolver::new());
    
    let result = resolver.lookup_domain("nonexistent.dark");
    assert!(matches!(result, Err(DarkResolverError::DomainNotFound)));
}

#[tokio::test]
async fn test_dark_domain_with_special_characters() {
    let resolver = Arc::new(DarkResolver::new());
    let address = NetworkAddress::new([192, 168, 1, 1], 8080);
    
    // Test valid domains with hyphens and numbers
    let valid_domains = vec![
        "test-service.dark",
        "service-123.dark",
        "123-service.dark",
        "multi-part-name.dark",
    ];
    
    for domain in valid_domains {
        let result = resolver.register_domain(domain, address.clone());
        assert!(result.is_ok(), "Domain '{}' should be valid", domain);
    }
}

#[tokio::test]
async fn test_dark_domain_case_sensitivity() {
    let resolver = Arc::new(DarkResolver::new());
    let address = NetworkAddress::new([10, 0, 0, 1], 8080);
    
    // Register lowercase domain
    resolver.register_domain("testcase.dark", address.clone()).unwrap();
    
    // Try to register uppercase version (should be treated as different)
    let result = resolver.register_domain("TESTCASE.dark", address.clone());
    assert!(result.is_ok(), "Case-sensitive domains should be allowed");
    
    // Verify both exist
    assert!(resolver.lookup_domain("testcase.dark").is_ok());
    assert!(resolver.lookup_domain("TESTCASE.dark").is_ok());
}

#[tokio::test]
async fn test_dark_domain_thread_safety() {
    let resolver = Arc::new(DarkResolver::new());
    let barrier = Arc::new(tokio::sync::Barrier::new(5));
    let mut handles = vec![];
    
    // Test concurrent reads and writes
    for i in 0..5 {
        let resolver_clone = resolver.clone();
        let barrier_clone = barrier.clone();
        
        let handle = tokio::spawn(async move {
            // Wait for all threads to be ready
            barrier_clone.wait().await;
            
            if i % 2 == 0 {
                // Register domains
                let domain = format!("thread-{}.dark", i);
                let address = NetworkAddress::new([10, 0, 0, i as u8], 9000 + i as u16);
                resolver_clone.register_domain(&domain, address).unwrap();
            } else {
                // Lookup domains
                for j in 0..i {
                    let domain = format!("thread-{}.dark", j);
                    let _ = resolver_clone.lookup_domain(&domain);
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }
}