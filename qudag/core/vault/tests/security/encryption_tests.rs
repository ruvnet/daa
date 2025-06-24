use qudag_vault_core::*;
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[cfg(test)]
mod encryption_security_tests {
    use super::*;

    #[test]
    fn test_vault_key_derivation_strength() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let password = "TestPassword123!";

        let start = Instant::now();
        let vault = Vault::create(vault_path.to_str().unwrap(), password).unwrap();
        let duration = start.elapsed();

        // Argon2id should take noticeable time (>100ms) for key derivation
        assert!(
            duration > Duration::from_millis(100),
            "Key derivation too fast, may be insecure: {:?}",
            duration
        );

        // But not too slow for usability
        assert!(
            duration < Duration::from_secs(5),
            "Key derivation too slow: {:?}",
            duration
        );
    }

    #[test]
    fn test_encrypted_vault_file_content() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let password = "SuperSecretPassword";

        let mut vault = Vault::create(vault_path.to_str().unwrap(), password).unwrap();
        vault
            .add_secret("test/secret", "testuser", Some("MySecretPassword"))
            .unwrap();
        drop(vault);

        // Read raw vault file
        let vault_contents = std::fs::read(&vault_path).unwrap();
        let vault_str = String::from_utf8_lossy(&vault_contents);

        // Ensure no plaintext passwords in file
        assert!(
            !vault_str.contains("MySecretPassword"),
            "Password found in plaintext!"
        );
        assert!(
            !vault_str.contains("testuser"),
            "Username found in plaintext!"
        );
        assert!(
            !vault_str.contains("test/secret"),
            "Label found in plaintext!"
        );
        assert!(
            !vault_str.contains(password),
            "Master password found in plaintext!"
        );
    }

    #[test]
    fn test_memory_zeroization() {
        use zeroize::Zeroize;

        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        // Test that sensitive data is zeroized
        {
            let mut password = String::from("SensitivePassword123!");
            let password_ptr = password.as_ptr();
            let password_len = password.len();

            let _vault = Vault::create(vault_path.to_str().unwrap(), &password).unwrap();

            // Zeroize password
            password.zeroize();

            // Verify memory was cleared (best effort check)
            unsafe {
                let slice = std::slice::from_raw_parts(password_ptr, password_len);
                assert!(
                    slice.iter().all(|&b| b == 0),
                    "Password memory not zeroized!"
                );
            }
        }
    }

    #[test]
    fn test_timing_attack_resistance() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let correct_password = "CorrectPassword123!";

        // Create vault
        let _vault = Vault::create(vault_path.to_str().unwrap(), correct_password).unwrap();
        drop(_vault);

        // Test multiple incorrect passwords with similar prefixes
        let test_passwords = vec![
            "WrongPassword123!",
            "CrongPassword123!",
            "CorrectPassword123",
            "CorrectPassword124!",
            "TotallyWrong",
        ];

        let mut durations = Vec::new();

        for wrong_password in test_passwords {
            let start = Instant::now();
            let _ = Vault::open(vault_path.to_str().unwrap(), wrong_password);
            let duration = start.elapsed();
            durations.push(duration);
        }

        // Calculate variance in timing
        let avg_duration: Duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        let max_deviation = durations
            .iter()
            .map(|d| {
                if *d > avg_duration {
                    *d - avg_duration
                } else {
                    avg_duration - *d
                }
            })
            .max()
            .unwrap();

        // Timing should be consistent (within 20% deviation)
        let twenty_percent = avg_duration / 5;
        assert!(
            max_deviation < twenty_percent,
            "Timing attack possible: max deviation {:?} exceeds 20% of avg {:?}",
            max_deviation,
            avg_duration
        );
    }
}

#[cfg(test)]
mod quantum_resistance_tests {
    use super::*;
    use qudag_crypto::{DilithiumKeyPair, KyberKeyPair};

    #[test]
    fn test_kyber_key_encapsulation() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        // Create vault with Kyber support
        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Generate Kyber keypair
        let keypair = KyberKeyPair::generate().unwrap();

        // Set vault public key
        vault.set_kyber_public_key(keypair.public()).unwrap();

        // Export vault key for recipient
        let recipient_keypair = KyberKeyPair::generate().unwrap();
        let encapsulated = vault
            .export_vault_key_for(&recipient_keypair.public())
            .unwrap();

        // Verify encapsulation size is correct for Kyber
        assert!(
            encapsulated.ciphertext.len() > 1000,
            "Kyber ciphertext too small"
        );
        assert!(
            encapsulated.ciphertext.len() < 2000,
            "Kyber ciphertext too large"
        );
    }

    #[test]
    fn test_dilithium_signature_verification() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");

        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();
        vault
            .add_secret("test/secret", "user", Some("pass"))
            .unwrap();

        // Generate Dilithium keypair
        let keypair = DilithiumKeyPair::generate().unwrap();

        // Sign vault export
        let export_data = vault.export_signed(&keypair).unwrap();

        // Verify signature
        let verified = Vault::verify_import(&export_data, &keypair.public()).unwrap();
        assert!(verified, "Dilithium signature verification failed");

        // Tamper with data
        let mut tampered_data = export_data.clone();
        tampered_data.data[0] ^= 0xFF;

        // Verification should fail
        let tampered_result = Vault::verify_import(&tampered_data, &keypair.public());
        assert!(
            tampered_result.is_err() || !tampered_result.unwrap(),
            "Tampered data verification should fail"
        );
    }

    #[test]
    fn test_quantum_safe_password_generation() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Generate quantum-safe password (using quantum RNG if available)
        let password = vault.generate_password(32, Charset::All).unwrap();

        // Verify entropy
        assert_eq!(password.len(), 32);

        // Calculate Shannon entropy
        let entropy = calculate_shannon_entropy(&password);
        assert!(
            entropy > 4.0,
            "Generated password has low entropy: {}",
            entropy
        );
    }
}

#[cfg(test)]
mod side_channel_tests {
    use super::*;

    #[test]
    fn test_constant_time_comparison() {
        // Test that password comparison is constant-time
        let password1 = "TestPassword123!";
        let password2 = "TestPassword123!";
        let password3 = "WrongPassword123";

        let mut timings = Vec::new();

        // Compare equal passwords multiple times
        for _ in 0..100 {
            let start = Instant::now();
            let _ = constant_time_compare(password1.as_bytes(), password2.as_bytes());
            timings.push(start.elapsed());
        }

        // Compare different passwords
        for _ in 0..100 {
            let start = Instant::now();
            let _ = constant_time_compare(password1.as_bytes(), password3.as_bytes());
            timings.push(start.elapsed());
        }

        // Verify timing consistency
        let avg_timing = timings.iter().sum::<Duration>() / timings.len() as u32;
        let max_deviation = timings
            .iter()
            .map(|t| {
                if *t > avg_timing {
                    *t - avg_timing
                } else {
                    avg_timing - *t
                }
            })
            .max()
            .unwrap();

        assert!(
            max_deviation < avg_timing / 10,
            "Timing not constant: max deviation {:?}",
            max_deviation
        );
    }

    #[test]
    fn test_cache_timing_resistance() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test_vault.qdag");
        let mut vault = Vault::create(vault_path.to_str().unwrap(), "TestPassword").unwrap();

        // Add many secrets to stress cache
        for i in 0..1000 {
            vault
                .add_secret(&format!("test/secret{}", i), "user", Some("pass"))
                .unwrap();
        }

        // Access patterns should not reveal information through cache timing
        let labels = vec![
            "test/secret0",
            "test/secret500",
            "test/secret999",
            "nonexistent",
        ];
        let mut timings = Vec::new();

        for label in &labels {
            let start = Instant::now();
            let _ = vault.get_secret(label);
            timings.push(start.elapsed());
        }

        // Verify no significant timing differences
        let avg_timing = timings.iter().take(3).sum::<Duration>() / 3;
        for timing in &timings {
            let deviation = if *timing > avg_timing {
                *timing - avg_timing
            } else {
                avg_timing - *timing
            };
            assert!(
                deviation < avg_timing / 2,
                "Cache timing leak detected: {:?} deviates from avg {:?}",
                timing,
                avg_timing
            );
        }
    }
}

// Helper function to calculate Shannon entropy
fn calculate_shannon_entropy(s: &str) -> f64 {
    use std::collections::HashMap;

    let mut char_counts = HashMap::new();
    for c in s.chars() {
        *char_counts.entry(c).or_insert(0) += 1;
    }

    let len = s.len() as f64;
    char_counts
        .values()
        .map(|&count| {
            let p = count as f64 / len;
            -p * p.log2()
        })
        .sum()
}

// Mock constant-time comparison (would be implemented in actual vault)
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    result == 0
}
