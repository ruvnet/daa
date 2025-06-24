//! Unit tests for dark resolver module

use qudag_network::dark_resolver::{DarkDomainRecord, DarkResolver, DarkResolverError};
use qudag_network::shadow_address::{ShadowAddress, NetworkType};
use qudag_network::types::PeerId;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

#[tokio::test]
async fn test_dark_resolver_basic() {
    let mut resolver = DarkResolver::new();
    
    // Create domain record
    let domain = "hidden.dark".to_string();
    let shadow_addr = ShadowAddress::generate(NetworkType::Mainnet);
    let record = DarkDomainRecord {
        domain: domain.clone(),
        shadow_address: shadow_addr.clone(),
        peer_id: PeerId::random(),
        ip_addresses: vec![
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
        ],
        last_seen: SystemTime::now(),
        metadata: HashMap::new(),
    };
    
    // Register domain
    assert!(resolver.register_domain(record.clone()).await.is_ok());
    
    // Resolve domain
    let resolved = resolver.resolve_domain(&domain).await;
    assert!(resolved.is_ok());
    
    let result = resolved.unwrap();
    assert_eq!(result.domain, domain);
    assert_eq!(result.shadow_address, shadow_addr);
}

#[tokio::test]
async fn test_dark_resolver_not_found() {
    let resolver = DarkResolver::new();
    
    let result = resolver.resolve_domain("nonexistent.dark").await;
    assert!(matches!(result, Err(DarkResolverError::DomainNotFound)));
}

#[tokio::test]
async fn test_dark_resolver_update() {
    let mut resolver = DarkResolver::new();
    let domain = "update.dark".to_string();
    
    // Initial registration
    let mut record = DarkDomainRecord {
        domain: domain.clone(),
        shadow_address: ShadowAddress::generate(NetworkType::Mainnet),
        peer_id: PeerId::random(),
        ip_addresses: vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))],
        last_seen: SystemTime::now(),
        metadata: HashMap::new(),
    };
    
    resolver.register_domain(record.clone()).await.unwrap();
    
    // Update with new IP
    record.ip_addresses.push(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)));
    record.last_seen = SystemTime::now();
    
    assert!(resolver.update_domain(record.clone()).await.is_ok());
    
    // Verify update
    let resolved = resolver.resolve_domain(&domain).await.unwrap();
    assert_eq!(resolved.ip_addresses.len(), 2);
}

#[tokio::test]
async fn test_dark_resolver_ttl_expiry() {
    let mut resolver = DarkResolver::with_ttl(Duration::from_millis(100));
    
    let record = DarkDomainRecord {
        domain: "expire.dark".to_string(),
        shadow_address: ShadowAddress::generate(NetworkType::Mainnet),
        peer_id: PeerId::random(),
        ip_addresses: vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))],
        last_seen: SystemTime::now(),
        metadata: HashMap::new(),
    };
    
    resolver.register_domain(record).await.unwrap();
    
    // Should resolve immediately
    assert!(resolver.resolve_domain("expire.dark").await.is_ok());
    
    // Wait for TTL to expire
    sleep(Duration::from_millis(150)).await;
    
    // Should be expired
    let result = resolver.resolve_domain("expire.dark").await;
    assert!(matches!(result, Err(DarkResolverError::DomainExpired)));
}

#[tokio::test]
async fn test_dark_resolver_metadata() {
    let mut resolver = DarkResolver::new();
    
    let mut metadata = HashMap::new();
    metadata.insert("protocol".to_string(), "qudag/1.0".to_string());
    metadata.insert("capabilities".to_string(), "quantum,onion".to_string());
    
    let record = DarkDomainRecord {
        domain: "meta.dark".to_string(),
        shadow_address: ShadowAddress::generate(NetworkType::Testnet),
        peer_id: PeerId::random(),
        ip_addresses: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
        last_seen: SystemTime::now(),
        metadata: metadata.clone(),
    };
    
    resolver.register_domain(record).await.unwrap();
    
    let resolved = resolver.resolve_domain("meta.dark").await.unwrap();
    assert_eq!(resolved.metadata.get("protocol"), Some(&"qudag/1.0".to_string()));
    assert_eq!(resolved.metadata.get("capabilities"), Some(&"quantum,onion".to_string()));
}

#[tokio::test]
async fn test_dark_resolver_multiple_domains() {
    let mut resolver = DarkResolver::new();
    
    // Register multiple domains
    for i in 0..10 {
        let record = DarkDomainRecord {
            domain: format!("site{}.dark", i),
            shadow_address: ShadowAddress::generate(NetworkType::Mainnet),
            peer_id: PeerId::random(),
            ip_addresses: vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, i as u8))],
            last_seen: SystemTime::now(),
            metadata: HashMap::new(),
        };
        resolver.register_domain(record).await.unwrap();
    }
    
    // List all domains
    let domains = resolver.list_domains().await;
    assert_eq!(domains.len(), 10);
    
    // Resolve specific ones
    for i in [0, 5, 9] {
        let domain = format!("site{}.dark", i);
        let resolved = resolver.resolve_domain(&domain).await.unwrap();
        assert_eq!(resolved.domain, domain);
    }
}

#[tokio::test]
async fn test_dark_resolver_shadow_address_lookup() {
    let mut resolver = DarkResolver::new();
    let shadow_addr = ShadowAddress::generate(NetworkType::Mainnet);
    
    let record = DarkDomainRecord {
        domain: "shadow.dark".to_string(),
        shadow_address: shadow_addr.clone(),
        peer_id: PeerId::random(),
        ip_addresses: vec![IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))],
        last_seen: SystemTime::now(),
        metadata: HashMap::new(),
    };
    
    resolver.register_domain(record).await.unwrap();
    
    // Resolve by shadow address
    let resolved = resolver.resolve_by_shadow_address(&shadow_addr).await;
    assert!(resolved.is_ok());
    assert_eq!(resolved.unwrap().domain, "shadow.dark");
}

#[tokio::test]
async fn test_dark_resolver_peer_id_lookup() {
    let mut resolver = DarkResolver::new();
    let peer_id = PeerId::random();
    
    // Register multiple domains for same peer
    for i in 0..3 {
        let record = DarkDomainRecord {
            domain: format!("peer{}.dark", i),
            shadow_address: ShadowAddress::generate(NetworkType::Mainnet),
            peer_id,
            ip_addresses: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, i as u8))],
            last_seen: SystemTime::now(),
            metadata: HashMap::new(),
        };
        resolver.register_domain(record).await.unwrap();
    }
    
    // Find all domains for peer
    let domains = resolver.find_domains_by_peer(&peer_id).await;
    assert_eq!(domains.len(), 3);
}

#[tokio::test]
async fn test_dark_resolver_remove_domain() {
    let mut resolver = DarkResolver::new();
    let domain = "remove.dark";
    
    let record = DarkDomainRecord {
        domain: domain.to_string(),
        shadow_address: ShadowAddress::generate(NetworkType::Mainnet),
        peer_id: PeerId::random(),
        ip_addresses: vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))],
        last_seen: SystemTime::now(),
        metadata: HashMap::new(),
    };
    
    resolver.register_domain(record).await.unwrap();
    assert!(resolver.resolve_domain(domain).await.is_ok());
    
    // Remove domain
    assert!(resolver.remove_domain(domain).await.is_ok());
    
    // Should not resolve anymore
    assert!(matches!(
        resolver.resolve_domain(domain).await,
        Err(DarkResolverError::DomainNotFound)
    ));
}

#[tokio::test]
async fn test_dark_resolver_concurrent_access() {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let resolver = Arc::new(Mutex::new(DarkResolver::new()));
    let mut handles = vec![];
    
    // Concurrent registrations
    for i in 0..20 {
        let resolver_clone = Arc::clone(&resolver);
        let handle = tokio::spawn(async move {
            let record = DarkDomainRecord {
                domain: format!("concurrent{}.dark", i),
                shadow_address: ShadowAddress::generate(NetworkType::Mainnet),
                peer_id: PeerId::random(),
                ip_addresses: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, i as u8))],
                last_seen: SystemTime::now(),
                metadata: HashMap::new(),
            };
            resolver_clone.lock().await.register_domain(record).await
        });
        handles.push(handle);
    }
    
    // Wait for all registrations
    let results: Vec<_> = futures::future::join_all(handles).await;
    assert!(results.iter().all(|r| r.is_ok()));
    
    // Verify all domains registered
    let domains = resolver.lock().await.list_domains().await;
    assert_eq!(domains.len(), 20);
}

#[tokio::test]
async fn test_dark_resolver_validation() {
    let mut resolver = DarkResolver::new();
    
    // Test invalid domain names
    let invalid_domains = vec![
        "",
        "no-extension",
        ".dark",
        "double..dark",
        "spaces in name.dark",
        "very-long-domain-name-that-exceeds-reasonable-limits-for-dark-web-addressing.dark",
    ];
    
    for invalid in invalid_domains {
        let record = DarkDomainRecord {
            domain: invalid.to_string(),
            shadow_address: ShadowAddress::generate(NetworkType::Mainnet),
            peer_id: PeerId::random(),
            ip_addresses: vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))],
            last_seen: SystemTime::now(),
            metadata: HashMap::new(),
        };
        
        let result = resolver.register_domain(record).await;
        assert!(matches!(result, Err(DarkResolverError::InvalidDomain)));
    }
}

#[tokio::test]
async fn test_dark_resolver_cleanup() {
    let mut resolver = DarkResolver::with_ttl(Duration::from_millis(50));
    
    // Register domains with different timestamps
    let now = SystemTime::now();
    let old_time = now - Duration::from_secs(3600); // 1 hour ago
    
    let old_record = DarkDomainRecord {
        domain: "old.dark".to_string(),
        shadow_address: ShadowAddress::generate(NetworkType::Mainnet),
        peer_id: PeerId::random(),
        ip_addresses: vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))],
        last_seen: old_time,
        metadata: HashMap::new(),
    };
    
    let new_record = DarkDomainRecord {
        domain: "new.dark".to_string(),
        shadow_address: ShadowAddress::generate(NetworkType::Mainnet),
        peer_id: PeerId::random(),
        ip_addresses: vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2))],
        last_seen: now,
        metadata: HashMap::new(),
    };
    
    resolver.register_domain(old_record).await.unwrap();
    resolver.register_domain(new_record).await.unwrap();
    
    // Run cleanup
    resolver.cleanup_expired().await;
    
    // Old should be gone, new should remain
    assert!(matches!(
        resolver.resolve_domain("old.dark").await,
        Err(DarkResolverError::DomainNotFound)
    ));
    assert!(resolver.resolve_domain("new.dark").await.is_ok());
}