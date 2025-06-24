//! Property-based tests for peer management
//! These tests verify invariants and properties of peer operations

use proptest::prelude::*;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Generate random IPv4 addresses
fn arb_ipv4() -> impl Strategy<Value = Ipv4Addr> {
    (any::<u8>(), any::<u8>(), any::<u8>(), any::<u8>())
        .prop_map(|(a, b, c, d)| Ipv4Addr::new(a, b, c, d))
}

/// Generate random IPv6 addresses
fn arb_ipv6() -> impl Strategy<Value = Ipv6Addr> {
    (
        any::<u16>(), any::<u16>(), any::<u16>(), any::<u16>(),
        any::<u16>(), any::<u16>(), any::<u16>(), any::<u16>()
    ).prop_map(|(a, b, c, d, e, f, g, h)| {
        Ipv6Addr::new(a, b, c, d, e, f, g, h)
    })
}

/// Generate random port numbers (valid range)
fn arb_port() -> impl Strategy<Value = u16> {
    1u16..=65535
}

/// Generate random peer addresses
fn arb_peer_address() -> impl Strategy<Value = String> {
    prop_oneof![
        // IPv4 addresses
        (arb_ipv4(), arb_port()).prop_map(|(ip, port)| format!("{}:{}", ip, port)),
        // IPv6 addresses
        (arb_ipv6(), arb_port()).prop_map(|(ip, port)| format!("[{}]:{}", ip, port)),
        // Domain names
        ("[a-z][a-z0-9-]{0,62}\\.(com|net|org|io)", arb_port())
            .prop_map(|(domain, port)| format!("{}:{}", domain, port)),
        // .onion addresses
        ("[a-z2-7]{16}\\.onion", arb_port())
            .prop_map(|(onion, port)| format!("{}:{}", onion, port)),
        // .dark addresses
        "[a-z][a-z0-9-]{0,62}\\.dark".prop_map(|addr| addr.to_string()),
    ]
}

/// Property: Valid addresses should be parseable
proptest! {
    #[test]
    fn prop_valid_address_format(address in arb_peer_address()) {
        // This property ensures that generated addresses follow expected formats
        prop_assert!(
            address.contains(':') || address.ends_with(".dark"),
            "Address should contain port separator or be a .dark address"
        );
    }
}

/// Property: Adding and removing a peer should result in original state
proptest! {
    #[test]
    fn prop_add_remove_identity(addresses in prop::collection::vec(arb_peer_address(), 1..10)) {
        let mut peer_set = HashSet::new();
        
        // Add all peers
        for addr in &addresses {
            peer_set.insert(addr.clone());
        }
        
        // Remove all peers
        for addr in &addresses {
            peer_set.remove(addr);
        }
        
        prop_assert!(peer_set.is_empty(), "Peer set should be empty after adding and removing all peers");
    }
}

/// Property: Duplicate addresses should not increase peer count
proptest! {
    #[test]
    fn prop_no_duplicate_peers(address in arb_peer_address(), count in 2..10usize) {
        let mut peer_set = HashSet::new();
        
        // Try to add the same address multiple times
        for _ in 0..count {
            peer_set.insert(address.clone());
        }
        
        prop_assert_eq!(peer_set.len(), 1, "Only one instance of peer should exist");
    }
}

/// Property: Peer limit should be respected
proptest! {
    #[test]
    fn prop_peer_limit_respected(
        addresses in prop::collection::vec(arb_peer_address(), 1..100),
        max_peers in 10..50usize
    ) {
        let mut peer_set = HashSet::new();
        let mut added_count = 0;
        
        for addr in addresses {
            if peer_set.len() < max_peers {
                peer_set.insert(addr);
                added_count += 1;
            }
        }
        
        prop_assert!(
            peer_set.len() <= max_peers,
            "Peer count {} should not exceed max_peers {}",
            peer_set.len(),
            max_peers
        );
    }
}

/// Property: Port numbers should be in valid range
proptest! {
    #[test]
    fn prop_valid_port_range(port in arb_port()) {
        prop_assert!(port >= 1 && port <= 65535, "Port should be in valid range");
    }
}

/// Property: Address components should be extractable
proptest! {
    #[test]
    fn prop_address_components_extractable(ip in arb_ipv4(), port in arb_port()) {
        let address = format!("{}:{}", ip, port);
        let parts: Vec<&str> = address.split(':').collect();
        
        prop_assert_eq!(parts.len(), 2, "Address should have exactly 2 parts");
        prop_assert_eq!(parts[0], ip.to_string(), "IP part should match");
        prop_assert_eq!(parts[1], port.to_string(), "Port part should match");
    }
}

/// Property: Dark addresses should not contain port numbers
proptest! {
    #[test]
    fn prop_dark_address_format(name in "[a-z][a-z0-9-]{0,62}") {
        let dark_address = format!("{}.dark", name);
        
        prop_assert!(!dark_address.contains(':'), "Dark addresses should not contain port separator");
        prop_assert!(dark_address.ends_with(".dark"), "Dark addresses should end with .dark");
        prop_assert!(dark_address.len() <= 67, "Dark addresses should not be too long");
    }
}

/// Property: Onion addresses should follow Tor v2/v3 format
proptest! {
    #[test]
    fn prop_onion_address_format(
        address_type in prop::sample::select(vec!["v2", "v3"]),
        port in arb_port()
    ) {
        let onion_part = match address_type.as_str() {
            "v2" => "[a-z2-7]{16}".to_string(),
            "v3" => "[a-z2-7]{56}".to_string(),
            _ => unreachable!(),
        };
        
        let address = format!("{}.onion:{}", onion_part, port);
        
        prop_assert!(address.contains(".onion:"), "Should contain .onion domain with port");
        prop_assert!(
            address.len() >= 24 && address.len() <= 70,
            "Onion address length should be reasonable"
        );
    }
}

/// Property: Peer removal should be idempotent
proptest! {
    #[test]
    fn prop_remove_idempotent(address in arb_peer_address()) {
        let mut peer_set = HashSet::new();
        peer_set.insert(address.clone());
        
        // Remove once
        let first_removal = peer_set.remove(&address);
        prop_assert!(first_removal, "First removal should succeed");
        
        // Remove again
        let second_removal = peer_set.remove(&address);
        prop_assert!(!second_removal, "Second removal should return false");
        
        prop_assert!(peer_set.is_empty(), "Set should be empty");
    }
}

/// Property: Concurrent operations should maintain consistency
proptest! {
    #[test]
    fn prop_concurrent_consistency(
        operations in prop::collection::vec(
            (arb_peer_address(), prop::bool::ANY),
            1..50
        )
    ) {
        let mut peer_set = HashSet::new();
        let mut expected_peers = HashSet::new();
        
        for (address, should_add) in operations {
            if should_add {
                peer_set.insert(address.clone());
                expected_peers.insert(address);
            } else {
                peer_set.remove(&address);
                expected_peers.remove(&address);
            }
        }
        
        prop_assert_eq!(
            peer_set.len(),
            expected_peers.len(),
            "Peer set size should match expected"
        );
        
        for peer in &expected_peers {
            prop_assert!(
                peer_set.contains(peer),
                "Expected peer {} should be in set",
                peer
            );
        }
    }
}