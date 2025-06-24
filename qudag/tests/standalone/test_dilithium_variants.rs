use pqcrypto_dilithium::{dilithium2, dilithium3, dilithium5};
use pqcrypto_traits::sign::{PublicKey, SecretKey};

fn main() {
    println!("Testing Dilithium parameter sets...");
    
    // Test Dilithium2 (ML-DSA-44)
    let (pk2, sk2) = dilithium2::keypair();
    println!("Dilithium2 (ML-DSA-44): PK size = {}, SK size = {}", 
             pk2.as_bytes().len(), sk2.as_bytes().len());
    
    // Test Dilithium3 (ML-DSA-65)
    let (pk3, sk3) = dilithium3::keypair();
    println!("Dilithium3 (ML-DSA-65): PK size = {}, SK size = {}", 
             pk3.as_bytes().len(), sk3.as_bytes().len());
    
    // Test Dilithium5 (ML-DSA-87)
    let (pk5, sk5) = dilithium5::keypair();
    println!("Dilithium5 (ML-DSA-87): PK size = {}, SK size = {}", 
             pk5.as_bytes().len(), sk5.as_bytes().len());
             
    // Test signature sizes
    let message = b"test message";
    let sig2 = dilithium2::sign(message, &sk2);
    let sig3 = dilithium3::sign(message, &sk3);
    let sig5 = dilithium5::sign(message, &sk5);
    
    println!("Signature sizes:");
    println!("  Dilithium2: {}", sig2.as_bytes().len());
    println!("  Dilithium3: {}", sig3.as_bytes().len());
    println!("  Dilithium5: {}", sig5.as_bytes().len());
}