# Security Threat Model for QuDAG WASM Vault

## Executive Summary

This document provides a comprehensive security threat model for the QuDAG WASM-based vault system. It identifies potential threats, analyzes attack vectors, and proposes mitigation strategies specific to WebAssembly environments. The analysis covers cryptographic implementations, memory safety, side-channel attacks, and post-quantum readiness.

## Table of Contents

1. [Threat Landscape Overview](#threat-landscape-overview)
2. [WASM-Specific Security Considerations](#wasm-specific-security-considerations)
3. [Memory Protection Strategies](#memory-protection-strategies)
4. [Secure Random Number Generation](#secure-random-number-generation)
5. [Key Derivation Security](#key-derivation-security)
6. [Side-Channel Attack Analysis](#side-channel-attack-analysis)
7. [Post-Quantum Cryptography Readiness](#post-quantum-cryptography-readiness)
8. [Attack Scenarios and Mitigations](#attack-scenarios-and-mitigations)
9. [Security Architecture Patterns](#security-architecture-patterns)
10. [Compliance and Standards](#compliance-and-standards)

## Threat Landscape Overview

### Primary Threat Actors

1. **Nation-State Actors**
   - Motivation: Intelligence gathering, mass surveillance
   - Capabilities: Zero-day exploits, advanced persistent threats
   - Risk Level: HIGH

2. **Cybercriminal Organizations**
   - Motivation: Financial gain through data theft
   - Capabilities: Ransomware, credential harvesting
   - Risk Level: HIGH

3. **Malicious Insiders**
   - Motivation: Data exfiltration, sabotage
   - Capabilities: Privileged access, social engineering
   - Risk Level: MEDIUM

4. **Script Kiddies**
   - Motivation: Notoriety, vandalism
   - Capabilities: Known exploits, automated tools
   - Risk Level: LOW

### Attack Surface Analysis

```
┌─────────────────────────────────────────────┐
│            Browser Environment              │
├─────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────────┐   │
│  │ JavaScript  │───▶│   WASM Module   │   │
│  │   Context   │    │  (Vault Core)   │   │
│  └─────────────┘    └─────────────────┘   │
│         │                    │             │
│  ┌──────▼────────┐  ┌───────▼─────────┐  │
│  │ DOM Interface │  │ Linear Memory   │  │
│  └───────────────┘  └─────────────────┘  │
│                              │             │
│  ┌───────────────────────────▼─────────┐  │
│  │        Browser Storage APIs         │  │
│  │  (IndexedDB, LocalStorage, etc.)   │  │
│  └─────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
                      │
                 Network I/O
                      │
              ┌───────▼────────┐
              │ Remote Servers │
              └────────────────┘
```

### Threat Categories

1. **Cryptographic Attacks**
   - Weak randomness exploitation
   - Side-channel attacks (timing, power, EM)
   - Cryptanalysis of implementations

2. **Memory Safety Violations**
   - Buffer overflows in WASM linear memory
   - Use-after-free vulnerabilities
   - Information leakage through uninitialized memory

3. **Injection Attacks**
   - Cross-site scripting (XSS)
   - WASM module tampering
   - Supply chain attacks

4. **Authentication Bypass**
   - Password brute-forcing
   - Session hijacking
   - Multi-factor authentication bypass

5. **Data Exfiltration**
   - Memory scraping
   - Network interception
   - Persistent storage compromise

## WASM-Specific Security Considerations

### WASM Security Model

WebAssembly provides several security guarantees:

1. **Memory Isolation**: Linear memory is sandboxed
2. **Type Safety**: Strong typing prevents many vulnerabilities
3. **Control Flow Integrity**: No arbitrary jumps or function pointers
4. **Deterministic Execution**: Predictable behavior across platforms

### WASM Limitations and Risks

1. **No Direct System Access**
   - Cannot access /dev/urandom directly
   - No memory protection (mlock) capabilities
   - Limited to browser-provided APIs

2. **Observable Memory**
   - Host can inspect entire linear memory
   - No hardware-enforced memory protection
   - Potential for memory scraping attacks

3. **Timing Precision**
   - High-resolution timers enable timing attacks
   - Spectre/Meltdown vulnerabilities in some contexts
   - Cross-origin timing leaks

### Security Boundaries

```rust
pub struct WASMSecurityBoundary {
    // WASM module cannot access these directly
    host_memory: HostMemory,
    system_resources: SystemResources,
    other_origins: CrossOriginData,
    
    // WASM module has access to these
    linear_memory: LinearMemory,
    imported_functions: ImportedFunctions,
    table_elements: TableElements,
}

impl WASMSecurityBoundary {
    pub fn validate_import(&self, import: &Import) -> Result<(), SecurityError> {
        // Validate all imported functions
        match import {
            Import::Function(name, signature) => {
                if !self.is_allowed_import(name) {
                    return Err(SecurityError::UnauthorizedImport);
                }
                if !self.validate_signature(signature) {
                    return Err(SecurityError::InvalidSignature);
                }
            }
            Import::Memory(limits) => {
                if limits.maximum > MAX_MEMORY_PAGES {
                    return Err(SecurityError::ExcessiveMemory);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Memory Protection Strategies

### Linear Memory Security

WASM's linear memory model presents unique challenges:

```rust
pub struct SecureMemoryManager {
    // Memory regions for different security levels
    public_region: MemoryRegion,
    sensitive_region: MemoryRegion,
    crypto_region: MemoryRegion,
    
    // Guard pages between regions
    guard_pages: Vec<GuardPage>,
    
    // Memory access tracking
    access_log: AccessLog,
}

impl SecureMemoryManager {
    pub fn allocate_sensitive(&mut self, size: usize) -> Result<SecurePtr, MemoryError> {
        // Allocate in sensitive region
        let ptr = self.sensitive_region.allocate(size)?;
        
        // Clear memory before use
        unsafe {
            ptr::write_bytes(ptr.as_mut_ptr(), 0, size);
        }
        
        // Set up guard pages
        self.protect_with_guards(ptr, size)?;
        
        // Track allocation
        self.access_log.record_allocation(ptr, size);
        
        Ok(SecurePtr::new(ptr, size))
    }
    
    fn protect_with_guards(&mut self, ptr: *mut u8, size: usize) -> Result<(), MemoryError> {
        // Add guard pages before and after allocation
        let page_size = 4096;
        let start = (ptr as usize / page_size) * page_size;
        let end = ((ptr as usize + size + page_size - 1) / page_size) * page_size;
        
        // Mark guard pages
        self.guard_pages.push(GuardPage {
            start: start - page_size,
            end: start,
        });
        self.guard_pages.push(GuardPage {
            start: end,
            end: end + page_size,
        });
        
        Ok(())
    }
}
```

### Memory Scrubbing

Implement secure memory cleanup:

```rust
pub struct MemoryScrubber {
    scrub_pattern: [u8; 16],
    verification_rounds: usize,
}

impl MemoryScrubber {
    pub fn scrub_memory(&self, ptr: *mut u8, len: usize) {
        unsafe {
            // Multiple overwrite passes with different patterns
            for round in 0..self.verification_rounds {
                // Pattern 1: All zeros
                ptr::write_bytes(ptr, 0x00, len);
                compiler_fence(Ordering::SeqCst);
                
                // Pattern 2: All ones
                ptr::write_bytes(ptr, 0xFF, len);
                compiler_fence(Ordering::SeqCst);
                
                // Pattern 3: Random pattern
                for i in 0..len {
                    ptr.add(i).write_volatile(self.scrub_pattern[i % 16]);
                }
                compiler_fence(Ordering::SeqCst);
                
                // Pattern 4: Complement of random pattern
                for i in 0..len {
                    ptr.add(i).write_volatile(!self.scrub_pattern[i % 16]);
                }
                compiler_fence(Ordering::SeqCst);
            }
            
            // Final zero pass
            ptr::write_bytes(ptr, 0x00, len);
            compiler_fence(Ordering::SeqCst);
        }
    }
}

// Automatic scrubbing on drop
pub struct SensitiveBuffer {
    data: Vec<u8>,
    scrubber: MemoryScrubber,
}

impl Drop for SensitiveBuffer {
    fn drop(&mut self) {
        self.scrubber.scrub_memory(
            self.data.as_mut_ptr(),
            self.data.len()
        );
    }
}
```

### Stack Protection

Protect sensitive data on the stack:

```rust
#[repr(C)]
pub struct StackProtector<T> {
    canary_start: u64,
    data: T,
    canary_end: u64,
}

impl<T> StackProtector<T> {
    const CANARY_VALUE: u64 = 0xDEADBEEF_CAFEBABE;
    
    pub fn new(data: T) -> Self {
        Self {
            canary_start: Self::CANARY_VALUE,
            data,
            canary_end: Self::CANARY_VALUE,
        }
    }
    
    pub fn verify(&self) -> Result<&T, SecurityError> {
        if self.canary_start != Self::CANARY_VALUE || 
           self.canary_end != Self::CANARY_VALUE {
            // Stack corruption detected
            panic!("Stack corruption detected!");
        }
        Ok(&self.data)
    }
}

// Use in sensitive functions
pub fn process_password(password: &str) -> Result<Key, Error> {
    let protected_password = StackProtector::new(password.to_string());
    
    // Process with protection
    let key = derive_key(protected_password.verify()?)?;
    
    // Stack automatically cleaned on drop
    Ok(key)
}
```

## Secure Random Number Generation

### WASM RNG Architecture

```rust
pub struct SecureRandom {
    // Primary entropy source
    crypto_rng: Box<dyn CryptoRng>,
    
    // Backup entropy sources
    timestamp_entropy: TimestampEntropy,
    user_entropy: UserEntropy,
    
    // Entropy pool
    entropy_pool: EntropyPool,
    
    // Health monitoring
    health_monitor: RNGHealthMonitor,
}

impl SecureRandom {
    pub fn new() -> Result<Self, RNGError> {
        // Try WebCrypto first
        let crypto_rng: Box<dyn CryptoRng> = if let Ok(web_crypto) = WebCryptoRNG::new() {
            Box::new(web_crypto)
        } else {
            // Fallback to WASM implementation
            Box::new(ChaChaRng::from_entropy())
        };
        
        Ok(Self {
            crypto_rng,
            timestamp_entropy: TimestampEntropy::new(),
            user_entropy: UserEntropy::new(),
            entropy_pool: EntropyPool::new(),
            health_monitor: RNGHealthMonitor::new(),
        })
    }
    
    pub fn generate(&mut self, output: &mut [u8]) -> Result<(), RNGError> {
        // Check RNG health
        if !self.health_monitor.is_healthy() {
            return Err(RNGError::UnhealthyRNG);
        }
        
        // Mix multiple entropy sources
        let mut mixed_entropy = vec![0u8; output.len()];
        
        // Primary source
        self.crypto_rng.fill_bytes(&mut mixed_entropy)?;
        
        // Mix in additional entropy
        let timestamp_entropy = self.timestamp_entropy.get_entropy(output.len());
        for (i, &byte) in timestamp_entropy.iter().enumerate() {
            mixed_entropy[i] ^= byte;
        }
        
        // Mix in user entropy if available
        if let Some(user_entropy) = self.user_entropy.get_entropy(output.len()) {
            for (i, &byte) in user_entropy.iter().enumerate() {
                mixed_entropy[i] ^= byte;
            }
        }
        
        // Update entropy pool
        self.entropy_pool.mix(&mixed_entropy);
        
        // Extract output
        self.entropy_pool.extract(output);
        
        // Update health metrics
        self.health_monitor.record_generation(output.len());
        
        Ok(())
    }
}
```

### Entropy Sources

```rust
pub struct WebCryptoRNG;

impl WebCryptoRNG {
    pub fn new() -> Result<Self, RNGError> {
        // Check if crypto.getRandomValues is available
        let crypto = web_sys::window()
            .ok_or(RNGError::NoCryptoAPI)?
            .crypto()
            .map_err(|_| RNGError::NoCryptoAPI)?;
            
        Ok(Self)
    }
    
    pub fn fill_bytes(&self, dest: &mut [u8]) -> Result<(), RNGError> {
        let array = js_sys::Uint8Array::new_with_length(dest.len() as u32);
        
        web_sys::window()
            .unwrap()
            .crypto()
            .unwrap()
            .get_random_values_with_u8_array(&array)
            .map_err(|_| RNGError::GenerationFailed)?;
            
        array.copy_to(dest);
        Ok(())
    }
}

pub struct TimestampEntropy {
    last_timestamp: u64,
    counter: u64,
}

impl TimestampEntropy {
    pub fn get_entropy(&mut self, len: usize) -> Vec<u8> {
        let mut entropy = Vec::with_capacity(len);
        
        for _ in 0..len {
            let now = js_sys::Date::now() as u64;
            let jitter = now.wrapping_sub(self.last_timestamp);
            self.last_timestamp = now;
            self.counter = self.counter.wrapping_add(1);
            
            // Mix timestamp jitter with counter
            let mixed = jitter ^ self.counter.rotate_left(32);
            entropy.push((mixed & 0xFF) as u8);
        }
        
        entropy
    }
}
```

### RNG Health Monitoring

```rust
pub struct RNGHealthMonitor {
    // Statistical tests
    monobit_test: MonobitTest,
    runs_test: RunsTest,
    entropy_estimator: EntropyEstimator,
    
    // Failure tracking
    consecutive_failures: usize,
    total_failures: usize,
    
    // Configuration
    max_consecutive_failures: usize,
}

impl RNGHealthMonitor {
    pub fn is_healthy(&self) -> bool {
        self.consecutive_failures < self.max_consecutive_failures
    }
    
    pub fn test_output(&mut self, data: &[u8]) -> bool {
        // Run NIST SP 800-90B tests
        let monobit_pass = self.monobit_test.test(data);
        let runs_pass = self.runs_test.test(data);
        let entropy_ok = self.entropy_estimator.estimate(data) > 7.0; // bits per byte
        
        let all_pass = monobit_pass && runs_pass && entropy_ok;
        
        if all_pass {
            self.consecutive_failures = 0;
        } else {
            self.consecutive_failures += 1;
            self.total_failures += 1;
        }
        
        all_pass
    }
}
```

## Key Derivation Security

### Argon2id Implementation

```rust
pub struct SecureArgon2 {
    // Argon2id parameters
    memory_cost: u32,
    time_cost: u32,
    parallelism: u32,
    
    // Side-channel protections
    memory_scrambler: MemoryScrambler,
    timing_randomizer: TimingRandomizer,
}

impl SecureArgon2 {
    pub fn derive_key(
        &self,
        password: &[u8],
        salt: &[u8],
        output_len: usize,
    ) -> Result<Vec<u8>, KDFError> {
        // Input validation
        if password.len() == 0 || password.len() > 4096 {
            return Err(KDFError::InvalidPassword);
        }
        if salt.len() < 16 {
            return Err(KDFError::InvalidSalt);
        }
        
        // Allocate secure memory
        let memory_blocks = self.memory_cost as usize * 1024;
        let mut memory = SecureMemory::new(memory_blocks * 1024);
        
        // Initialize with password and salt
        self.initialize_memory(&mut memory, password, salt)?;
        
        // Main loop with timing randomization
        for pass in 0..self.time_cost {
            self.timing_randomizer.add_jitter();
            
            // Process memory blocks
            for slice in 0..4 {
                for lane in 0..self.parallelism {
                    self.process_segment(&mut memory, pass, slice, lane)?;
                }
            }
            
            // Scramble memory layout (side-channel protection)
            self.memory_scrambler.scramble(&mut memory);
        }
        
        // Extract output
        let output = self.finalize(&memory, output_len)?;
        
        // Explicit cleanup
        memory.scrub();
        
        Ok(output)
    }
    
    fn process_segment(
        &self,
        memory: &mut SecureMemory,
        pass: u32,
        slice: u32,
        lane: u32,
    ) -> Result<(), KDFError> {
        // Argon2 compression function with constant-time operations
        let segment_length = memory.len() / (self.parallelism as usize * 4);
        
        for index in 0..segment_length {
            // Reference block indices
            let j1 = self.index_alpha(pass, slice, index as u32, lane);
            let j2 = self.index_beta(pass, slice, index as u32, lane);
            
            // Compression
            let block = self.compress(
                memory.get_block(j1)?,
                memory.get_block(j2)?
            )?;
            
            // Update memory
            memory.set_block(index, block)?;
        }
        
        Ok(())
    }
}
```

### Key Stretching Security

```rust
pub struct KeyStretcher {
    // Multiple KDF rounds for defense in depth
    primary_kdf: SecureArgon2,
    secondary_kdf: PBKDF2,
    
    // Entropy mixing
    entropy_mixer: EntropyMixer,
}

impl KeyStretcher {
    pub fn stretch_key(
        &self,
        weak_key: &[u8],
        context: &[u8],
    ) -> Result<StrongKey, StretchError> {
        // Round 1: Argon2id
        let intermediate1 = self.primary_kdf.derive_key(
            weak_key,
            context,
            64, // 512 bits
        )?;
        
        // Mix in additional entropy
        let mixed = self.entropy_mixer.mix(&intermediate1)?;
        
        // Round 2: PBKDF2-HMAC-SHA512
        let intermediate2 = self.secondary_kdf.derive_key(
            &mixed,
            context,
            100_000, // iterations
            64,
        )?;
        
        // Final key derivation with domain separation
        let mut final_key = [0u8; 32];
        let mut hasher = Blake3::new_keyed(&intermediate2[..32]);
        hasher.update(b"qudag-vault-master-key");
        hasher.update(context);
        hasher.finalize_xof().fill(&mut final_key);
        
        // Clean up intermediates
        drop(intermediate1);
        drop(mixed);
        drop(intermediate2);
        
        Ok(StrongKey::new(final_key))
    }
}
```

## Side-Channel Attack Analysis

### Timing Attack Mitigation

```rust
pub struct ConstantTimeOperations;

impl ConstantTimeOperations {
    #[inline(never)] // Prevent inlining for consistent timing
    pub fn compare(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        
        let mut result = 0u8;
        for i in 0..a.len() {
            result |= a[i] ^ b[i];
        }
        
        // Constant-time zero check
        result == 0
    }
    
    #[inline(never)]
    pub fn select<T: Copy>(a: T, b: T, condition: bool) -> T {
        // Branch-free selection
        let mask = (condition as u8).wrapping_neg();
        let a_bytes = unsafe { 
            slice::from_raw_parts(&a as *const T as *const u8, size_of::<T>()) 
        };
        let b_bytes = unsafe { 
            slice::from_raw_parts(&b as *const T as *const u8, size_of::<T>()) 
        };
        
        let mut result_bytes = vec![0u8; size_of::<T>()];
        for i in 0..size_of::<T>() {
            result_bytes[i] = (a_bytes[i] & mask) | (b_bytes[i] & !mask);
        }
        
        unsafe { ptr::read(result_bytes.as_ptr() as *const T) }
    }
    
    pub fn memory_access_pattern_hiding<T, F>(
        data: &[T],
        index: usize,
        f: F,
    ) -> T
    where
        T: Copy + Default,
        F: Fn(&T) -> T,
    {
        // Access all elements to hide the real target
        let mut result = T::default();
        
        for (i, item) in data.iter().enumerate() {
            let is_target = Self::compare(
                &i.to_le_bytes(),
                &index.to_le_bytes()
            );
            let processed = f(item);
            result = Self::select(processed, result, is_target);
        }
        
        result
    }
}
```

### Cache Attack Protection

```rust
pub struct CacheProtection {
    // Cache line size (typically 64 bytes)
    cache_line_size: usize,
    
    // Prefetch distance
    prefetch_distance: usize,
}

impl CacheProtection {
    pub fn protected_table_lookup<T: Copy>(
        &self,
        table: &[T],
        index: usize,
    ) -> T {
        // Ensure table is cache-aligned
        assert!(table.as_ptr() as usize % self.cache_line_size == 0);
        
        // Prefetch to confuse cache timing
        for i in 0..table.len() {
            unsafe {
                // Prefetch each cache line
                let ptr = table.as_ptr().add(i);
                core::arch::wasm32::memory_prefetch_temporal_l1(ptr as *const u8);
            }
        }
        
        // Perform actual lookup with dummy operations
        let mut result = table[0];
        for (i, &value) in table.iter().enumerate() {
            let is_target = constant_time_eq(i, index);
            result = constant_time_select(value, result, is_target);
        }
        
        result
    }
    
    pub fn scatter_gather_protection<T: Copy>(
        &self,
        data: &mut [T],
        operation: impl Fn(&mut T),
    ) {
        // Randomize access pattern
        let mut indices: Vec<usize> = (0..data.len()).collect();
        
        // Fisher-Yates shuffle
        for i in (1..indices.len()).rev() {
            let j = self.random_index(i + 1);
            indices.swap(i, j);
        }
        
        // Access in randomized order
        for &index in &indices {
            operation(&mut data[index]);
            
            // Add noise accesses
            for _ in 0..self.prefetch_distance {
                let noise_index = self.random_index(data.len());
                let _ = &data[noise_index]; // Dummy read
            }
        }
    }
}
```

### Power Analysis Defense

```rust
pub struct PowerAnalysisDefense {
    noise_generator: NoiseGenerator,
    power_balancer: PowerBalancer,
}

impl PowerAnalysisDefense {
    pub fn protected_operation<F, R>(&self, sensitive_op: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Start noise generation
        let noise_handle = self.noise_generator.start();
        
        // Balance power consumption
        let balanced_op = || {
            let result = sensitive_op();
            
            // Dummy operations to maintain constant power
            self.power_balancer.balance();
            
            result
        };
        
        // Execute with protection
        let result = balanced_op();
        
        // Stop noise generation
        noise_handle.stop();
        
        result
    }
}

pub struct NoiseGenerator {
    dummy_computations: Vec<Box<dyn Fn()>>,
}

impl NoiseGenerator {
    pub fn start(&self) -> NoiseHandle {
        // Spawn noise generation in background
        let handle = spawn_local(async {
            loop {
                // Random dummy computations
                for computation in &self.dummy_computations {
                    computation();
                    
                    // Random delay
                    let delay = Duration::from_micros(rand::random::<u64>() % 1000);
                    sleep(delay).await;
                }
            }
        });
        
        NoiseHandle { handle }
    }
}
```

## Post-Quantum Cryptography Readiness

### Hybrid Cryptographic Schemes

```rust
pub struct HybridCrypto {
    classical: ClassicalCrypto,
    post_quantum: PostQuantumCrypto,
}

impl HybridCrypto {
    pub async fn encrypt(
        &self,
        plaintext: &[u8],
        recipient_key: &HybridPublicKey,
    ) -> Result<HybridCiphertext, CryptoError> {
        // Generate ephemeral keys
        let classical_ephemeral = self.classical.generate_ephemeral()?;
        let pq_ephemeral = self.post_quantum.generate_ephemeral()?;
        
        // Key encapsulation (both classical and PQ)
        let (classical_shared, classical_encap) = self.classical
            .encapsulate(&recipient_key.classical, &classical_ephemeral)?;
            
        let (pq_shared, pq_encap) = self.post_quantum
            .encapsulate(&recipient_key.post_quantum, &pq_ephemeral)?;
            
        // Combine shared secrets
        let combined_key = self.combine_keys(&classical_shared, &pq_shared)?;
        
        // Encrypt with combined key
        let ciphertext = self.encrypt_data(&combined_key, plaintext)?;
        
        Ok(HybridCiphertext {
            classical_encapsulation: classical_encap,
            pq_encapsulation: pq_encap,
            ciphertext,
        })
    }
    
    fn combine_keys(&self, k1: &[u8], k2: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Use a KDF to combine keys securely
        let mut combined = Vec::with_capacity(k1.len() + k2.len());
        combined.extend_from_slice(k1);
        combined.extend_from_slice(k2);
        
        let mut output = vec![0u8; 32];
        let mut hasher = Blake3::new_derive_key("qudag-hybrid-key");
        hasher.update(&combined);
        hasher.finalize_xof().fill(&mut output);
        
        Ok(output)
    }
}
```

### ML-KEM (Kyber) Integration

```rust
pub struct MLKEMProvider {
    parameter_set: MLKEMParameterSet,
    side_channel_protection: bool,
}

impl MLKEMProvider {
    pub fn generate_keypair(&self) -> Result<(MLKEMPublicKey, MLKEMPrivateKey), MLKEMError> {
        // Generate random seed
        let mut seed = [0u8; 32];
        self.secure_random(&mut seed)?;
        
        // Key generation with side-channel protection
        let (pk, sk) = if self.side_channel_protection {
            self.protected_keygen(&seed)?
        } else {
            ml_kem_keygen(&seed, self.parameter_set)?
        };
        
        Ok((MLKEMPublicKey(pk), MLKEMPrivateKey(sk)))
    }
    
    pub fn encapsulate(
        &self,
        public_key: &MLKEMPublicKey,
    ) -> Result<(SharedSecret, Ciphertext), MLKEMError> {
        // Generate randomness
        let mut random = [0u8; 32];
        self.secure_random(&mut random)?;
        
        // Encapsulation with protection
        let (shared_secret, ciphertext) = if self.side_channel_protection {
            self.protected_encaps(&public_key.0, &random)?
        } else {
            ml_kem_encaps(&public_key.0, &random, self.parameter_set)?
        };
        
        Ok((SharedSecret(shared_secret), Ciphertext(ciphertext)))
    }
    
    fn protected_keygen(&self, seed: &[u8]) -> Result<(Vec<u8>, Vec<u8>), MLKEMError> {
        // Add noise and timing randomization
        with_side_channel_protection(|| {
            ml_kem_keygen(seed, self.parameter_set)
        })
    }
}
```

### ML-DSA (Dilithium) Integration

```rust
pub struct MLDSAProvider {
    parameter_set: MLDSAParameterSet,
    deterministic: bool,
}

impl MLDSAProvider {
    pub fn sign(
        &self,
        private_key: &MLDSAPrivateKey,
        message: &[u8],
    ) -> Result<MLDSASignature, MLDSAError> {
        // Add randomness for non-deterministic signing
        let randomness = if self.deterministic {
            None
        } else {
            let mut random = [0u8; 32];
            self.secure_random(&mut random)?;
            Some(random)
        };
        
        // Sign with side-channel protection
        let signature = with_side_channel_protection(|| {
            ml_dsa_sign(
                &private_key.0,
                message,
                randomness.as_ref(),
                self.parameter_set,
            )
        })?;
        
        Ok(MLDSASignature(signature))
    }
    
    pub fn verify(
        &self,
        public_key: &MLDSAPublicKey,
        message: &[u8],
        signature: &MLDSASignature,
    ) -> Result<bool, MLDSAError> {
        // Verification is typically not sensitive to side-channels
        ml_dsa_verify(
            &public_key.0,
            message,
            &signature.0,
            self.parameter_set,
        )
    }
}
```

### Migration Strategy

```rust
pub struct CryptoMigration {
    current_version: CryptoVersion,
    migration_plan: MigrationPlan,
}

impl CryptoMigration {
    pub fn migrate_vault(&self, vault: &mut Vault) -> Result<(), MigrationError> {
        match self.current_version {
            CryptoVersion::Classical => {
                // Phase 1: Add PQ alongside classical
                self.add_post_quantum_keys(vault)?;
                self.enable_hybrid_mode(vault)?;
            }
            CryptoVersion::Hybrid => {
                // Phase 2: Prepare for PQ-only
                self.verify_pq_readiness(vault)?;
                self.update_protocols(vault)?;
            }
            CryptoVersion::PostQuantum => {
                // Already migrated
                return Ok(());
            }
        }
        
        Ok(())
    }
    
    fn add_post_quantum_keys(&self, vault: &mut Vault) -> Result<(), MigrationError> {
        for entry in vault.entries_mut() {
            // Generate PQ keys alongside existing keys
            let pq_keypair = self.generate_pq_keypair()?;
            entry.add_pq_key(pq_keypair)?;
            
            // Re-encrypt with hybrid encryption
            entry.upgrade_encryption(EncryptionMode::Hybrid)?;
        }
        
        Ok(())
    }
}
```

## Attack Scenarios and Mitigations

### Scenario 1: Memory Scraping Attack

**Attack Description**: Malicious JavaScript attempts to read WASM linear memory to extract secrets.

**Mitigation Strategy**:

```rust
pub struct MemoryScrapingDefense {
    // Memory isolation
    sensitive_allocator: SensitiveAllocator,
    
    // Encryption at rest
    memory_encryptor: MemoryEncryptor,
    
    // Access control
    access_controller: MemoryAccessController,
}

impl MemoryScrapingDefense {
    pub fn protect_secret<T: Zeroize>(&self, secret: T) -> ProtectedSecret<T> {
        // Allocate in protected region
        let mut protected = self.sensitive_allocator.allocate::<T>();
        
        // Encrypt in memory
        let encrypted = self.memory_encryptor.encrypt(&secret);
        protected.store(encrypted);
        
        // Set access controls
        self.access_controller.restrict_access(&protected);
        
        // Clear original
        drop(secret); // Zeroize trait ensures cleanup
        
        ProtectedSecret { inner: protected }
    }
}
```

### Scenario 2: Timing Attack on Password Verification

**Attack Description**: Attacker measures time variations in password checking to guess password.

**Mitigation Strategy**:

```rust
pub struct TimingSafePasswordVerifier {
    // Constant-time comparison
    constant_time: ConstantTimeOperations,
    
    // Decoy operations
    decoy_generator: DecoyOperations,
}

impl TimingSafePasswordVerifier {
    pub fn verify(&self, input: &str, stored_hash: &[u8]) -> bool {
        // Always compute full hash
        let input_hash = self.compute_hash(input);
        
        // Add random delay
        let delay = self.random_delay();
        thread::sleep(delay);
        
        // Constant-time comparison
        let valid = self.constant_time.compare(&input_hash, stored_hash);
        
        // Execute decoy operations regardless of result
        self.decoy_generator.execute();
        
        // Return after fixed total time
        self.ensure_minimum_time();
        
        valid
    }
}
```

### Scenario 3: Cross-Site Scripting (XSS) Attack

**Attack Description**: Injected script attempts to access vault data.

**Mitigation Strategy**:

```rust
pub struct XSSDefense {
    csp_enforcer: ContentSecurityPolicy,
    dom_sanitizer: DOMSanitizer,
    script_blocker: ScriptBlocker,
}

impl XSSDefense {
    pub fn initialize(&self) {
        // Set strict CSP headers
        self.csp_enforcer.set_policy(
            "default-src 'self'; \
             script-src 'self' 'wasm-unsafe-eval'; \
             object-src 'none'; \
             base-uri 'self';"
        );
        
        // Block inline scripts
        self.script_blocker.block_inline_scripts();
        
        // Sanitize all DOM inputs
        self.dom_sanitizer.enable_auto_sanitization();
    }
    
    pub fn safe_render(&self, content: &str) -> SafeHTML {
        // Sanitize content
        let sanitized = self.dom_sanitizer.sanitize(content);
        
        // Additional validation
        if self.detect_script_injection(&sanitized) {
            return SafeHTML::empty();
        }
        
        SafeHTML::new(sanitized)
    }
}
```

### Scenario 4: Supply Chain Attack

**Attack Description**: Compromised dependency injects malicious code.

**Mitigation Strategy**:

```rust
pub struct SupplyChainDefense {
    // Dependency verification
    dependency_verifier: DependencyVerifier,
    
    // Runtime integrity checking
    integrity_monitor: IntegrityMonitor,
    
    // Sandboxing
    sandbox: WASMSandbox,
}

impl SupplyChainDefense {
    pub fn verify_dependencies(&self) -> Result<(), SecurityError> {
        // Check all dependencies
        for dep in self.get_dependencies() {
            // Verify signatures
            if !self.dependency_verifier.verify_signature(&dep) {
                return Err(SecurityError::InvalidDependency(dep.name));
            }
            
            // Check against known vulnerabilities
            if self.has_known_vulnerabilities(&dep) {
                return Err(SecurityError::VulnerableDependency(dep.name));
            }
            
            // Verify reproducible builds
            if !self.verify_reproducible_build(&dep) {
                return Err(SecurityError::NonReproducibleBuild(dep.name));
            }
        }
        
        Ok(())
    }
    
    pub fn runtime_protection(&self) {
        // Monitor for unexpected behavior
        self.integrity_monitor.start_monitoring();
        
        // Sandbox untrusted code
        self.sandbox.isolate_dependencies();
    }
}
```

## Security Architecture Patterns

### Defense in Depth

```rust
pub struct DefenseInDepth {
    layers: Vec<Box<dyn SecurityLayer>>,
}

impl DefenseInDepth {
    pub fn process_request(&self, request: Request) -> Result<Response, SecurityError> {
        let mut context = SecurityContext::new(request);
        
        // Process through each security layer
        for layer in &self.layers {
            match layer.process(&mut context) {
                SecurityDecision::Allow => continue,
                SecurityDecision::Deny(reason) => {
                    self.log_security_event(SecurityEvent::Denied {
                        layer: layer.name(),
                        reason,
                        context: context.clone(),
                    });
                    return Err(SecurityError::AccessDenied(reason));
                }
                SecurityDecision::Challenge(challenge) => {
                    return Ok(Response::Challenge(challenge));
                }
            }
        }
        
        // All layers passed
        Ok(Response::Success(context.into_response()))
    }
}

// Example layers
pub struct RateLimitLayer {
    limiter: RateLimiter,
}

pub struct AuthenticationLayer {
    authenticator: Authenticator,
}

pub struct AuthorizationLayer {
    authorizer: Authorizer,
}

pub struct AnomalyDetectionLayer {
    detector: AnomalyDetector,
}

pub struct EncryptionLayer {
    encryptor: Encryptor,
}
```

### Zero Trust Architecture

```rust
pub struct ZeroTrustVault {
    // Never trust, always verify
    verifier: TrustVerifier,
    
    // Continuous authentication
    continuous_auth: ContinuousAuth,
    
    // Micro-segmentation
    segmentation: MicroSegmentation,
}

impl ZeroTrustVault {
    pub async fn access_secret(
        &self,
        request: SecretAccessRequest,
    ) -> Result<Secret, AccessError> {
        // Verify device trust
        let device_trust = self.verifier.verify_device(&request.device_id).await?;
        if device_trust.score < 0.8 {
            return Err(AccessError::UntrustedDevice);
        }
        
        // Verify user identity
        let user_trust = self.verifier.verify_user(&request.user_id).await?;
        if user_trust.score < 0.9 {
            // Require additional authentication
            self.continuous_auth.challenge(&request.user_id).await?;
        }
        
        // Check network location
        let network_trust = self.verifier.verify_network(&request.source_ip).await?;
        if network_trust.score < 0.7 {
            return Err(AccessError::UntrustedNetwork);
        }
        
        // Access with minimal privileges
        let segment = self.segmentation.get_segment(&request.secret_id)?;
        segment.access_with_restrictions(request)
    }
}
```

## Compliance and Standards

### FIPS 140-3 Compliance

```rust
pub struct FIPS140_3Compliance {
    // Approved algorithms only
    approved_algorithms: HashSet<AlgorithmIdentifier>,
    
    // Self-tests
    self_test_suite: SelfTestSuite,
    
    // Audit logging
    audit_logger: AuditLogger,
}

impl FIPS140_3Compliance {
    pub fn validate_algorithm(&self, algo: &AlgorithmIdentifier) -> Result<(), ComplianceError> {
        if !self.approved_algorithms.contains(algo) {
            self.audit_logger.log_violation(
                ComplianceViolation::UnapprovedAlgorithm(algo.clone())
            );
            return Err(ComplianceError::UnapprovedAlgorithm);
        }
        Ok(())
    }
    
    pub fn run_self_tests(&self) -> Result<SelfTestReport, ComplianceError> {
        let mut report = SelfTestReport::new();
        
        // Known Answer Tests (KAT)
        report.add_result("AES-KAT", self.self_test_suite.aes_kat()?);
        report.add_result("SHA-KAT", self.self_test_suite.sha_kat()?);
        report.add_result("HMAC-KAT", self.self_test_suite.hmac_kat()?);
        report.add_result("ECDSA-KAT", self.self_test_suite.ecdsa_kat()?);
        
        // Conditional tests
        report.add_result("RNG-Health", self.self_test_suite.rng_health_test()?);
        report.add_result("Firmware-Integrity", self.self_test_suite.firmware_integrity()?);
        
        Ok(report)
    }
}
```

### GDPR Compliance

```rust
pub struct GDPRCompliance {
    // Data minimization
    data_minimizer: DataMinimizer,
    
    // Right to erasure
    data_eraser: DataEraser,
    
    // Data portability
    data_exporter: DataExporter,
    
    // Consent management
    consent_manager: ConsentManager,
}

impl GDPRCompliance {
    pub async fn handle_data_request(
        &self,
        request: DataSubjectRequest,
    ) -> Result<DataSubjectResponse, GDPRError> {
        // Verify identity
        let identity = self.verify_data_subject(&request).await?;
        
        match request.request_type {
            RequestType::Access => {
                let data = self.collect_user_data(&identity).await?;
                Ok(DataSubjectResponse::AccessData(data))
            }
            RequestType::Erasure => {
                self.data_eraser.erase_user_data(&identity).await?;
                Ok(DataSubjectResponse::ErasureComplete)
            }
            RequestType::Portability => {
                let export = self.data_exporter.export_user_data(&identity).await?;
                Ok(DataSubjectResponse::PortableData(export))
            }
            RequestType::Rectification => {
                self.rectify_user_data(&identity, &request.corrections).await?;
                Ok(DataSubjectResponse::RectificationComplete)
            }
        }
    }
}
```

### Security Audit Trail

```rust
pub struct SecurityAuditTrail {
    // Immutable log
    append_only_log: AppendOnlyLog,
    
    // Cryptographic proof
    merkle_tree: MerkleTree,
    
    // Time stamping
    time_stamp_authority: TimeStampAuthority,
}

impl SecurityAuditTrail {
    pub async fn log_security_event(&self, event: SecurityEvent) -> Result<AuditProof, AuditError> {
        // Create audit record
        let record = AuditRecord {
            timestamp: Utc::now(),
            event,
            context: self.capture_context(),
        };
        
        // Get timestamp proof
        let tsa_proof = self.time_stamp_authority.timestamp(&record).await?;
        
        // Append to log
        let log_index = self.append_only_log.append(&record).await?;
        
        // Update Merkle tree
        let merkle_proof = self.merkle_tree.add_leaf(&record)?;
        
        Ok(AuditProof {
            log_index,
            merkle_proof,
            tsa_proof,
            record_hash: hash(&record),
        })
    }
    
    pub fn verify_audit_trail(&self, proof: &AuditProof) -> Result<bool, AuditError> {
        // Verify Merkle proof
        if !self.merkle_tree.verify_proof(&proof.merkle_proof) {
            return Ok(false);
        }
        
        // Verify timestamp
        if !self.time_stamp_authority.verify(&proof.tsa_proof) {
            return Ok(false);
        }
        
        // Verify log integrity
        if !self.append_only_log.verify_entry(proof.log_index, &proof.record_hash) {
            return Ok(false);
        }
        
        Ok(true)
    }
}
```

## Conclusion

This comprehensive security threat model for the QuDAG WASM vault system addresses the unique challenges of implementing cryptographic operations in a browser environment. Key recommendations include:

1. **Memory Protection**: Implement multiple layers of memory protection including secure allocation, scrubbing, and access controls
2. **Cryptographic Security**: Use hybrid approaches combining WebCrypto API with pure WASM implementations
3. **Side-Channel Defense**: Apply constant-time operations, cache protection, and power analysis countermeasures
4. **Post-Quantum Readiness**: Implement hybrid cryptographic schemes to ensure future security
5. **Defense in Depth**: Layer multiple security controls to protect against various attack vectors
6. **Continuous Monitoring**: Implement comprehensive audit trails and anomaly detection

Regular security assessments and updates to this threat model are essential as the WASM ecosystem and threat landscape continue to evolve.