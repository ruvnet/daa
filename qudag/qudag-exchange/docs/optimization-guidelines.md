# QuDAG Exchange Optimization Guidelines

## For All Implementation Agents

This document provides optimization guidelines to ensure high performance from the start.

### General Principles

1. **Avoid Premature Pessimization**
   - Use `&str` instead of `String` where possible
   - Pass by reference instead of cloning
   - Use `Arc<T>` for shared immutable data
   - Prefer `Vec::with_capacity()` when size is known

2. **Concurrency First**
   - Use `DashMap` instead of `Mutex<HashMap>`
   - Prefer `parking_lot` mutexes over std mutexes
   - Design for parallel access from the start
   - Use `rayon` for CPU-bound parallel tasks

3. **Memory Efficiency**
   - Enable `#![forbid(unsafe_code)]` to ensure safety
   - Use `Box<[T]>` instead of `Vec<T>` for fixed-size arrays
   - Consider `SmallVec` for small collections
   - Zero-copy where possible with `Cow<T>`

### Module-Specific Guidelines

#### Ledger Implementation
```rust
// DO: Use concurrent data structures
use dashmap::DashMap;
pub struct Ledger {
    balances: DashMap<AccountId, Balance>,
}

// DON'T: Use sequential structures with locks
pub struct Ledger {
    balances: Mutex<HashMap<AccountId, Balance>>,
}
```

#### Transaction Processing
```rust
// DO: Batch verification
pub fn verify_batch(txs: &[Transaction]) -> Vec<bool> {
    txs.par_iter()
        .map(|tx| verify_single(tx))
        .collect()
}

// DON'T: Sequential verification
pub fn verify_batch(txs: &[Transaction]) -> Vec<bool> {
    txs.iter()
        .map(|tx| verify_single(tx))
        .collect()
}
```

#### Serialization
```rust
// DO: Use efficient binary formats
use bincode;
let bytes = bincode::serialize(&tx)?;

// DON'T: Use JSON for hot paths
let json = serde_json::to_string(&tx)?;
```

### Error Handling
```rust
// DO: Use zero-cost error types
#[derive(Debug, thiserror::Error)]
pub enum TransferError {
    #[error("Insufficient balance")]
    InsufficientBalance,
}

// DON'T: Use strings for errors
pub fn transfer() -> Result<(), String> {
    Err("Insufficient balance".to_string())
}
```

### Async Code
```rust
// DO: Use async for I/O bound operations
pub async fn fetch_balance(id: AccountId) -> Balance {
    // Async database query
}

// DON'T: Block the executor
pub async fn fetch_balance(id: AccountId) -> Balance {
    std::thread::sleep(Duration::from_secs(1)); // BLOCKS!
}
```

### WASM Considerations

1. **Feature Flags**
   ```toml
   [features]
   default = ["full"]
   wasm = ["wasm-bindgen", "getrandom/js"]
   full = ["tokio", "rocksdb"]
   ```

2. **Conditional Compilation**
   ```rust
   #[cfg(not(target_arch = "wasm32"))]
   use std::time::SystemTime;
   
   #[cfg(target_arch = "wasm32")]
   use web_sys::Date;
   ```

3. **Size Optimization**
   - Use `wee_alloc` for WASM builds
   - Avoid large dependencies in WASM
   - Tree-shake with `wasm-opt`

### Testing for Performance

Always include benchmarks with new features:

```rust
#[bench]
fn bench_transfer(b: &mut Bencher) {
    let ledger = setup_ledger();
    b.iter(|| {
        ledger.transfer(&from, &to, 100)
    });
}
```

### Profiling Checklist

Before considering a module complete:

- [ ] Run `cargo bench` and record baseline
- [ ] Profile with `cargo flamegraph`
- [ ] Check memory usage with `valgrind`
- [ ] Verify no performance regressions
- [ ] Document any optimization decisions

### Common Pitfalls to Avoid

1. **String Allocations**
   ```rust
   // BAD: Allocates on every call
   fn get_error() -> String {
       "Error occurred".to_string()
   }
   
   // GOOD: No allocation
   fn get_error() -> &'static str {
       "Error occurred"
   }
   ```

2. **Unnecessary Cloning**
   ```rust
   // BAD: Clones entire vector
   fn process(data: Vec<u8>) { }
   
   // GOOD: Borrows data
   fn process(data: &[u8]) { }
   ```

3. **Blocking in Async**
   ```rust
   // BAD: Blocks the executor
   async fn compute() {
       expensive_sync_operation();
   }
   
   // GOOD: Run in blocking pool
   async fn compute() {
       tokio::task::spawn_blocking(|| {
           expensive_sync_operation()
       }).await
   }
   ```

### Performance Targets

Every implementation should aim for:

- Ledger lookup: <1ms for 1M accounts
- Transaction verification: >10k/sec single-threaded
- Memory per account: <100 bytes
- WASM bundle: <500KB for core functionality

### Monitoring Integration

Include metrics in implementations:

```rust
use metrics::{counter, histogram};

pub fn transfer(from: &AccountId, to: &AccountId, amount: Balance) {
    let start = Instant::now();
    
    // ... transfer logic ...
    
    histogram!("transfer_duration", start.elapsed());
    counter!("transfers_total", 1);
}
```

## Conclusion

Following these guidelines ensures QuDAG Exchange achieves its performance targets. When in doubt, measure first with benchmarks, then optimize based on data.