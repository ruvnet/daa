// Simple test of a documentation example

fn main() {
    println!("Testing a simple documentation example...");
    
    // Test the Digest example
    let digest = Digest(vec![0x12, 0x34, 0x56, 0x78]);
    let bytes = digest.as_bytes();
    assert_eq!(bytes, &[0x12, 0x34, 0x56, 0x78]);
    println!("✓ Digest example works");
    
    // Test Digest into_bytes
    let digest2 = Digest(vec![0x12, 0x34, 0x56, 0x78]);
    let bytes2 = digest2.into_bytes();
    assert_eq!(bytes2, vec![0x12, 0x34, 0x56, 0x78]);
    println!("✓ Digest into_bytes example works");
    
    println!("All examples passed!");
}

// Copy the Digest struct for testing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest(Vec<u8>);

impl Digest {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}