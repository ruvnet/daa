# TDD Workflow

## Steps

### 1. RED Phase
- Write failing test first
- Test must compile but fail
- Document expected behavior
- Run test to verify failure

### 2. GREEN Phase 
- Write minimal code to pass test
- Focus on functionality, not elegance
- Run test to verify pass
- No premature optimization

### 3. REFACTOR Phase
- Improve code structure
- Maintain test passing
- Apply coding standards
- Optimize if needed

## Decision Gates
- RED: Test compiles and fails
- GREEN: Test passes, code works
- REFACTOR: Tests pass, code clean

## Success Criteria
- All tests pass
- Code meets standards
- Coverage targets met
- Performance acceptable

## Example
```rust
// RED: Write failing test
#[test]
fn test_ml_kem_keygen() {
    let keypair = MlKem::keygen();
    assert!(keypair.is_valid());
}

// GREEN: Minimal implementation
impl MlKem {
    pub fn keygen() -> Keypair {
        Keypair::generate()
    }
}

// REFACTOR: Improve implementation
impl MlKem {
    pub fn keygen() -> Result<Keypair, CryptoError> {
        let mut rng = ChaCha20Rng::from_entropy();
        Keypair::generate_with_rng(&mut rng)
    }
}