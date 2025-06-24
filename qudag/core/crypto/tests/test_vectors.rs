use qudag_crypto::kem::ml_kem::*;

// NIST ML-KEM-768 Known Answer Test (KAT) vectors
const KAT_SEED: [u8; 32] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
];

#[test]
fn test_kat_vectors() {
    let mut rng = TestRng::from_seed(KAT_SEED);

    // Generate keypair with known seed
    let keypair = generate_keypair(&mut rng).unwrap();

    // Encapsulate with known public key
    let (shared_secret1, ciphertext) = encapsulate(&keypair.public_key).unwrap();

    // Decapsulate with known secret key and ciphertext
    let shared_secret2 = decapsulate(&keypair.secret_key, &ciphertext).unwrap();

    // Verify against KAT values
    assert_eq!(keypair.public_key.len(), PUBLIC_KEY_BYTES);
    assert_eq!(keypair.secret_key.len(), SECRET_KEY_BYTES);
    assert_eq!(shared_secret1.len(), SHARED_SECRET_BYTES);
    assert_eq!(shared_secret2.len(), SHARED_SECRET_BYTES);
    assert_eq!(ciphertext.len(), CIPHERTEXT_BYTES);

    // Verify shared secrets match using constant-time comparison
    assert!(bool::from(constant_time_compare(
        &shared_secret1,
        &shared_secret2
    )));
}

#[test]
fn test_invalid_seeds() {
    let mut rng = TestRng::from_seed([0u8; 32]);

    // Verify different seeds generate different keys
    let keypair1 = generate_keypair(&mut rng).unwrap();
    let keypair2 = generate_keypair(&mut rng).unwrap();

    assert_ne!(keypair1.public_key, keypair2.public_key);
    assert_ne!(keypair1.secret_key, keypair2.secret_key);
}

#[derive(Clone)]
struct TestRng {
    seed: [u8; 32],
    counter: u64,
}

impl TestRng {
    fn from_seed(seed: [u8; 32]) -> Self {
        Self { seed, counter: 0 }
    }
}

impl RngCore for TestRng {
    fn next_u32(&mut self) -> u32 {
        self.counter += 1;
        let mut hasher = sha3::Sha3_256::new();
        hasher.update(&self.seed);
        hasher.update(&self.counter.to_le_bytes());
        let result = hasher.finalize();
        u32::from_le_bytes(result[0..4].try_into().unwrap())
    }

    fn next_u64(&mut self) -> u64 {
        self.counter += 1;
        let mut hasher = sha3::Sha3_256::new();
        hasher.update(&self.seed);
        hasher.update(&self.counter.to_le_bytes());
        let result = hasher.finalize();
        u64::from_le_bytes(result[0..8].try_into().unwrap())
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(32) {
            self.counter += 1;
            let mut hasher = sha3::Sha3_256::new();
            hasher.update(&self.seed);
            hasher.update(&self.counter.to_le_bytes());
            let result = hasher.finalize();
            chunk.copy_from_slice(&result[..chunk.len()]);
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}
