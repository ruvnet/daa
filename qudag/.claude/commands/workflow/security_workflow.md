# Security Workflow

## Steps

### 1. Static Analysis
- Run security linters
- Check unsafe blocks
- Validate memory safety
- Review error handling

### 2. Crypto Review
- Constant-time operations
- RNG implementation
- Key management
- Memory zeroization

### 3. Network Security
- Protocol validation
- Authentication flows
- Message integrity
- Privacy measures

### 4. Integration Security
- Component interactions
- Trust boundaries
- Attack surface
- Access controls

## Decision Gates
- No critical findings
- Memory safety verified
- Crypto operations valid
- Network security sound

## Success Criteria
- All checks pass
- No timing attacks
- Memory properly handled
- Components secure

## Example
```rust
// Security review checklist
impl MlKem {
    // 1. Constant-time operations
    // 2. Secure RNG usage
    // 3. Key zeroization
    // 4. Error handling
    pub fn encapsulate(pk: &PublicKey) -> Result<(SharedSecret, Ciphertext), CryptoError> {
        // Use constant-time ops
        let mut rng = ChaCha20Rng::from_entropy();
        let (ss, ct) = encaps_ct(pk, &mut rng)?;
        
        // Ensure cleanup
        defer! { ss.zeroize() };
        
        Ok((ss, ct))
    }
}