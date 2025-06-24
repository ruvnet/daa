// Test file to understand ml-kem API
use qudag_crypto::ml_kem::MlKem768;

fn main() {
    let mut rng = rand::thread_rng();

    // Generate key pair
    let (pk, sk) = MlKem768::generate_keypair(&mut rng);

    println!("Public key generated");
    println!("Secret key generated");

    // Encapsulate
    let (ciphertext, shared_secret1) = MlKem768::encapsulate(&pk, &mut rng);

    println!("Ciphertext generated");
    println!("Shared secret 1 generated");

    // Decapsulate
    let shared_secret2 = MlKem768::decapsulate(&ciphertext, &sk);

    println!("Shared secret 2 generated");
    println!("Shared secrets match: {}", shared_secret1 == shared_secret2);
}
