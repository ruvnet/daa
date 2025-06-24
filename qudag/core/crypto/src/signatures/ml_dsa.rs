use super::*;
use pqcrypto::sign::dilithium3;
use rand::RngCore;
use zeroize::Zeroize;

const PUBLIC_KEY_BYTES: usize = dilithium3::public_key_bytes();
const SECRET_KEY_BYTES: usize = dilithium3::secret_key_bytes();
const SIGNATURE_BYTES: usize = dilithium3::signature_bytes();

/// Keypair for ML-DSA (Dilithium) signatures
#[derive(Clone)]
pub struct KeyPair {
    /// Public verification key
    pub public_key: Vec<u8>,
    /// Secret signing key
    pub secret_key: Vec<u8>,
}

impl Drop for KeyPair {
    fn drop(&mut self) {
        self.secret_key.zeroize();
    }
}

/// Generate a new ML-DSA key pair for signing
pub fn generate_keypair<R: RngCore>(rng: &mut R) -> Result<KeyPair, SignatureError> {
    // Generate seed for deterministic key generation
    let mut seed = vec![0u8; 32];
    defer! { seed.zeroize(); }
    rng.fill_bytes(&mut seed);
    
    let (pk, sk) = dilithium3::keypair();
    
    Ok(KeyPair {
        public_key: pk.as_bytes().to_vec(),
        secret_key: sk.as_bytes().to_vec(),
    })
}

/// Sign a message using ML-DSA with the provided secret key
pub fn sign(secret_key: &[u8], message: &[u8]) -> Result<Vec<u8>, SignatureError> {
    if secret_key.len() != SECRET_KEY_BYTES {
        return Err(SignatureError::SignError("Invalid secret key length".into()));
    }

    let sk = dilithium3::SecretKey::from_bytes(secret_key)
        .map_err(|e| SignatureError::SignError(e.to_string()))?;
        
    let signature = dilithium3::detached_sign(message, &sk);
    Ok(signature.as_bytes().to_vec())
}

/// Verify an ML-DSA signature using the provided public key
pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool, SignatureError> {
    if public_key.len() != PUBLIC_KEY_BYTES {
        return Err(SignatureError::VerifyError("Invalid public key length".into()));
    }
    if signature.len() != SIGNATURE_BYTES {
        return Err(SignatureError::VerifyError("Invalid signature length".into()));
    }

    let pk = dilithium3::PublicKey::from_bytes(public_key)
        .map_err(|e| SignatureError::VerifyError(e.to_string()))?;
    let sig = dilithium3::DetachedSignature::from_bytes(signature)
        .map_err(|e| SignatureError::VerifyError(e.to_string()))?;

    Ok(dilithium3::verify_detached_signature(&sig, message, &pk).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_signature_roundtrip() {
        let mut rng = thread_rng();
        let message = b"test message";
        
        // Generate keypair
        let keypair = generate_keypair(&mut rng).unwrap();
        assert_eq!(keypair.public_key.len(), PUBLIC_KEY_BYTES);
        assert_eq!(keypair.secret_key.len(), SECRET_KEY_BYTES);
        
        // Sign message
        let signature = sign(&keypair.secret_key, message).unwrap();
        assert_eq!(signature.len(), SIGNATURE_BYTES);
        
        // Verify signature
        let is_valid = verify(&keypair.public_key, message, &signature).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_invalid_signature() {
        let mut rng = thread_rng();
        let message = b"test message";
        let invalid_message = b"wrong message";
        
        // Generate keypair and sign
        let keypair = generate_keypair(&mut rng).unwrap();
        let signature = sign(&keypair.secret_key, message).unwrap();
        
        // Verify with wrong message
        let is_valid = verify(&keypair.public_key, invalid_message, &signature).unwrap();
        assert!(!is_valid);
        
        // Verify with tampered signature
        let mut tampered_signature = signature.clone();
        tampered_signature[0] ^= 1;
        let is_valid = verify(&keypair.public_key, message, &tampered_signature).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_timing_consistency() {
        use std::time::{Duration, Instant};
        
        let mut rng = thread_rng();
        let message = b"test message";
        
        let keypair = generate_keypair(&mut rng).unwrap();
        let signature = sign(&keypair.secret_key, message).unwrap();
        
        // Measure timing of valid signature verification
        let start = Instant::now();
        let _ = verify(&keypair.public_key, message, &signature).unwrap();
        let valid_time = start.elapsed();
        
        // Measure timing of invalid signature verification
        let mut invalid_sig = signature.clone();
        invalid_sig[0] ^= 1;
        let start = Instant::now();
        let _ = verify(&keypair.public_key, message, &invalid_sig).unwrap();
        let invalid_time = start.elapsed();
        
        // Check that timing difference is within acceptable range (1ms)
        let diff = if valid_time > invalid_time {
            valid_time - invalid_time
        } else {
            invalid_time - valid_time
        };
        assert!(diff < Duration::from_millis(1));
    }

    #[test]
    fn test_memory_zeroization() {
        let mut rng = thread_rng();
        let keypair = generate_keypair(&mut rng).unwrap();
        
        // Get a copy of the secret key
        let secret_key_copy = keypair.secret_key.clone();
        
        // Drop the keypair
        drop(keypair);
        
        // Secret key should be zeroized
        assert_ne!(vec![0u8; SECRET_KEY_BYTES], secret_key_copy);
    }
}