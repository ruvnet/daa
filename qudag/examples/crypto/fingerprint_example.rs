//! Quantum Fingerprint Example
//!
//! This example demonstrates how to use quantum-resistant fingerprinting
//! for data integrity and identification in the QuDAG protocol.

use qudag_crypto::{fingerprint::{Fingerprint, FingerprintError}, ml_dsa::MlDsaPublicKey};
use rand::thread_rng;
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Quantum Fingerprint Example");
    println!("==============================");

    // Example 1: Basic Fingerprint Generation
    basic_fingerprint_example()?;

    // Example 2: Data Integrity Verification
    data_integrity_example()?;

    // Example 3: Fingerprint-based Deduplication
    deduplication_example()?;

    // Example 4: Performance Comparison
    performance_comparison()?;

    // Example 5: Batch Fingerprinting
    batch_fingerprinting_example()?;

    // Example 6: Custom Data Types
    custom_data_types_example()?;

    println!("\n✅ All fingerprint examples completed successfully!");
    Ok(())
}

/// Example 1: Basic Fingerprint Generation
///
/// This demonstrates creating fingerprints for various types of data.
fn basic_fingerprint_example() -> Result<(), FingerprintError> {
    println!("\n🔍 Example 1: Basic Fingerprint Generation");

    let mut rng = thread_rng();

    // Generate fingerprints for different data types
    let text_data = b"Hello, quantum world!";
    let binary_data = &[0u8, 1u8, 2u8, 255u8, 254u8, 253u8];
    let empty_data = &[];
    let large_data = &vec![42u8; 10000];

    // Create fingerprints
    let (text_fingerprint, text_pub_key) = Fingerprint::generate(text_data, &mut rng)?;
    let (binary_fingerprint, binary_pub_key) = Fingerprint::generate(binary_data, &mut rng)?;
    let (empty_fingerprint, empty_pub_key) = Fingerprint::generate(empty_data, &mut rng)?;
    let (large_fingerprint, large_pub_key) = Fingerprint::generate(large_data, &mut rng)?;

    println!(
        "   Text data fingerprint: {} (size: {} bytes)",
        hex::encode(&text_fingerprint.data()[..8]),
        text_fingerprint.data().len()
    );
    println!(
        "   Binary data fingerprint: {} (size: {} bytes)",
        hex::encode(&binary_fingerprint.data()[..8]),
        binary_fingerprint.data().len()
    );
    println!(
        "   Empty data fingerprint: {} (size: {} bytes)",
        hex::encode(&empty_fingerprint.data()[..8]),
        empty_fingerprint.data().len()
    );
    println!(
        "   Large data fingerprint: {} (size: {} bytes)",
        hex::encode(&large_fingerprint.data()[..8]),
        large_fingerprint.data().len()
    );

    // Verify fingerprints can be validated
    text_fingerprint.verify(&text_pub_key)?;
    binary_fingerprint.verify(&binary_pub_key)?;
    empty_fingerprint.verify(&empty_pub_key)?;
    large_fingerprint.verify(&large_pub_key)?;
    println!("   ✓ All fingerprints verified successfully");

    // Verify different data produces different fingerprints
    assert_ne!(text_fingerprint.data(), binary_fingerprint.data());
    println!("   ✓ Different data produces different fingerprints");

    Ok(())
}

/// Example 2: Data Integrity Verification
///
/// This shows how to use fingerprints to verify data integrity.
fn data_integrity_example() -> Result<(), FingerprintError> {
    println!("\n🛡️  Example 2: Data Integrity Verification");

    let mut rng = thread_rng();

    // Original document
    let document = b"Important legal document content...";

    // Create fingerprint
    let (fingerprint, public_key) = Fingerprint::generate(document, &mut rng)?;
    println!("   Document fingerprinted: {} bytes", document.len());
    println!("   Fingerprint: {}", hex::encode(&fingerprint.data()[..16]));

    // Simulate storing the fingerprint and public key
    let stored_fingerprint = fingerprint.data().to_vec();
    let stored_signature = fingerprint.signature().to_vec();
    let stored_public_key = public_key.as_bytes().to_vec();

    // Later: Verify the document hasn't been tampered with
    println!("\n   Verifying document integrity...");

    // Recreate fingerprint from stored data
    let retrieved_public_key = MlDsaPublicKey::from_bytes(&stored_public_key)?;

    // Verify the fingerprint signature
    match retrieved_public_key.verify(&stored_fingerprint, &stored_signature) {
        Ok(_) => println!("   ✅ Document integrity verified!"),
        Err(e) => println!("   ❌ Document integrity check failed: {:?}", e),
    }

    // Simulate document tampering
    let tampered_document = b"Important legal document content... MODIFIED!";
    let (tampered_fingerprint, _) = Fingerprint::generate(tampered_document, &mut rng)?;

    if tampered_fingerprint.data() != stored_fingerprint {
        println!("   ✓ Tampering detected! Fingerprints don't match");
    }

    Ok(())
}

/// Example 3: Fingerprint-based Deduplication
///
/// This demonstrates using fingerprints for efficient deduplication.
fn deduplication_example() -> Result<(), FingerprintError> {
    println!("\n📦 Example 3: Fingerprint-based Deduplication");

    let mut rng = thread_rng();

    // Simulate a storage system
    let mut storage: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
    let mut fingerprint_index: HashMap<Vec<u8>, MlDsaPublicKey> = HashMap::new();

    // Files to store (some are duplicates)
    let files = vec![
        ("file1.txt", b"Content of file 1"),
        ("file2.txt", b"Content of file 2"),
        ("file3.txt", b"Content of file 1"), // Duplicate of file1
        ("file4.txt", b"Content of file 4"),
        ("file5.txt", b"Content of file 2"), // Duplicate of file2
    ];

    let mut stored_count = 0;
    let mut duplicate_count = 0;

    let files_len = files.len();
    for (filename, content) in files {
        let (fingerprint, public_key) = Fingerprint::generate(content, &mut rng)?;
        let fp_data = fingerprint.data().to_vec();

        if storage.contains_key(&fp_data) {
            println!(
                "   {} is a duplicate (fingerprint already exists)",
                filename
            );
            duplicate_count += 1;
        } else {
            storage.insert(fp_data.clone(), content.to_vec());
            fingerprint_index.insert(fp_data, public_key);
            println!(
                "   {} stored with fingerprint: {}",
                filename,
                hex::encode(&fingerprint.data()[..8])
            );
            stored_count += 1;
        }
    }

    println!("\n   Storage statistics:");
    println!("   - Files processed: {}", files_len);
    println!("   - Unique files stored: {}", stored_count);
    println!("   - Duplicates detected: {}", duplicate_count);
    println!(
        "   - Storage saved: {}%",
        (duplicate_count * 100) / files_len
    );

    Ok(())
}

/// Example 4: Performance Comparison
///
/// This compares fingerprint generation performance for different data sizes.
fn performance_comparison() -> Result<(), FingerprintError> {
    println!("\n⚡ Example 4: Performance Comparison");

    let mut rng = thread_rng();

    let data_sizes = vec![
        ("1 KB", 1024),
        ("10 KB", 10 * 1024),
        ("100 KB", 100 * 1024),
        ("1 MB", 1024 * 1024),
    ];

    for (label, size) in data_sizes {
        let data = vec![0u8; size];

        let start = Instant::now();
        let (fingerprint, public_key) = Fingerprint::generate(&data, &mut rng)?;
        let generation_time = start.elapsed();

        let start = Instant::now();
        fingerprint.verify(&public_key)?;
        let verification_time = start.elapsed();

        println!("   {} data:", label);
        println!("     - Generation time: {:?}", generation_time);
        println!("     - Verification time: {:?}", verification_time);
        println!(
            "     - Throughput: {:.2} MB/s",
            size as f64 / generation_time.as_secs_f64() / 1_048_576.0
        );
    }

    Ok(())
}

/// Example 5: Batch Fingerprinting
///
/// This demonstrates efficient batch processing of fingerprints.
fn batch_fingerprinting_example() -> Result<(), FingerprintError> {
    println!("\n📊 Example 5: Batch Fingerprinting");

    let mut rng = thread_rng();

    // Generate batch of messages
    let messages: Vec<Vec<u8>> = (0..100)
        .map(|i| {
            format!(
                "Message {}: Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
                i
            )
            .into_bytes()
        })
        .collect();

    println!("   Processing {} messages...", messages.len());

    let start = Instant::now();
    let mut fingerprints = Vec::new();
    let mut public_keys = Vec::new();

    for message in &messages {
        let (fp, pk) = Fingerprint::generate(message, &mut rng)?;
        fingerprints.push(fp);
        public_keys.push(pk);
    }

    let batch_time = start.elapsed();

    // Verify all fingerprints
    let start = Instant::now();
    for (fp, pk) in fingerprints.iter().zip(public_keys.iter()) {
        fp.verify(pk)?;
    }
    let verify_time = start.elapsed();

    println!("   Batch generation completed in {:?}", batch_time);
    println!(
        "   Average per fingerprint: {:?}",
        batch_time / messages.len() as u32
    );
    println!("   Batch verification completed in {:?}", verify_time);
    println!(
        "   Average per verification: {:?}",
        verify_time / messages.len() as u32
    );

    // Check uniqueness
    let unique_fingerprints: std::collections::HashSet<_> =
        fingerprints.iter().map(|fp| fp.data().to_vec()).collect();

    println!(
        "   All {} fingerprints are unique: {}",
        messages.len(),
        unique_fingerprints.len() == messages.len()
    );

    Ok(())
}

/// Example 6: Custom Data Types
///
/// This shows how to fingerprint custom data structures.
fn custom_data_types_example() -> Result<(), FingerprintError> {
    println!("\n🔧 Example 6: Custom Data Types");

    let mut rng = thread_rng();

    // Example: Fingerprinting a transaction
    #[derive(Debug)]
    struct Transaction {
        from: String,
        to: String,
        amount: u64,
        timestamp: u64,
    }

    impl Transaction {
        fn to_bytes(&self) -> Vec<u8> {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(self.from.as_bytes());
            bytes.extend_from_slice(self.to.as_bytes());
            bytes.extend_from_slice(&self.amount.to_le_bytes());
            bytes.extend_from_slice(&self.timestamp.to_le_bytes());
            bytes
        }

        fn fingerprint(
            &self,
            rng: &mut (impl rand::RngCore + rand::CryptoRng),
        ) -> Result<(Fingerprint, MlDsaPublicKey), FingerprintError> {
            Fingerprint::generate(&self.to_bytes(), rng)
        }
    }

    let tx = Transaction {
        from: "Alice".to_string(),
        to: "Bob".to_string(),
        amount: 1000,
        timestamp: 1234567890,
    };

    let (tx_fingerprint, tx_pub_key) = tx.fingerprint(&mut rng)?;
    println!(
        "   Transaction fingerprint: {}",
        hex::encode(&tx_fingerprint.data()[..16])
    );

    // Example: Fingerprinting a block
    #[derive(Debug)]
    struct Block {
        height: u64,
        prev_hash: Vec<u8>,
        transactions: Vec<Transaction>,
    }

    impl Block {
        fn to_bytes(&self) -> Vec<u8> {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&self.height.to_le_bytes());
            bytes.extend_from_slice(&self.prev_hash);
            for tx in &self.transactions {
                bytes.extend_from_slice(&tx.to_bytes());
            }
            bytes
        }

        fn fingerprint(
            &self,
            rng: &mut (impl rand::RngCore + rand::CryptoRng),
        ) -> Result<(Fingerprint, MlDsaPublicKey), FingerprintError> {
            Fingerprint::generate(&self.to_bytes(), rng)
        }
    }

    let block = Block {
        height: 100,
        prev_hash: vec![0u8; 32],
        transactions: vec![tx],
    };

    let (block_fingerprint, block_pub_key) = block.fingerprint(&mut rng)?;
    println!(
        "   Block fingerprint: {}",
        hex::encode(&block_fingerprint.data()[..16])
    );

    // Verify both fingerprints
    tx_fingerprint.verify(&tx_pub_key)?;
    block_fingerprint.verify(&block_pub_key)?;
    println!("   ✅ Custom data type fingerprints verified!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_fingerprint() {
        assert!(basic_fingerprint_example().is_ok());
    }

    #[test]
    fn test_data_integrity() {
        assert!(data_integrity_example().is_ok());
    }

    #[test]
    fn test_deduplication() {
        assert!(deduplication_example().is_ok());
    }

    #[test]
    fn test_batch_fingerprinting() {
        assert!(batch_fingerprinting_example().is_ok());
    }

    #[test]
    fn test_custom_data_types() {
        assert!(custom_data_types_example().is_ok());
    }
}
