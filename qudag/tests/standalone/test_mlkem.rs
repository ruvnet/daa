use qudag_crypto::ml_kem::MlKem768;
use qudag_crypto::kem::KeyEncapsulation;

fn main() {
    // Generate a keypair
    let (public_key, secret_key) = MlKem768::keygen().unwrap();
    
    println!("Public key size: {} bytes", public_key.as_bytes().len());
    println!("Secret key size: {} bytes", secret_key.as_bytes().len());
    
    // Encapsulate
    let (ciphertext, shared_secret) = MlKem768::encapsulate(&public_key).unwrap();
    
    println!("Ciphertext size: {} bytes", ciphertext.as_bytes().len());
    println!("Shared secret size: {} bytes", shared_secret.as_bytes().len());
    
    // Decapsulate
    let shared_secret2 = MlKem768::decapsulate(&secret_key, &ciphertext).unwrap();
    
    println!("Decapsulated shared secret size: {} bytes", shared_secret2.as_bytes().len());
    
    // Check if they match
    if shared_secret.as_bytes() == shared_secret2.as_bytes() {
        println!("✓ Shared secrets match!");
    } else {
        println!("✗ Shared secrets don't match!");
    }
}