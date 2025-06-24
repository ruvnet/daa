// Test file to understand ml-kem API
use ml_kem::MlKem768;

fn main() {
    let mut rng = rand::thread_rng();
    
    // Generate key pair
    let (dk, ek) = MlKem768::generate(&mut rng);
    
    println!("Decapsulation key length: {}", dk.as_bytes().len());
    println!("Encapsulation key length: {}", ek.as_bytes().len());
    
    // Encapsulate
    let (ct, ss1) = ek.encapsulate(&mut rng).unwrap();
    
    println!("Ciphertext length: {}", ct.as_bytes().len());
    println!("Shared secret length: {}", ss1.as_bytes().len());
    
    // Decapsulate
    let ss2 = dk.decapsulate(&ct).unwrap();
    
    println!("Shared secrets match: {}", ss1 == ss2);
}