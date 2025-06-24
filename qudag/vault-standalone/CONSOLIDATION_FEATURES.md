# Vault Consolidation - Unique Features from qudag-vault

## Features to Integrate into core/vault

### 1. Advanced DAG Operations (from dag.rs)
- **Cycle Detection**: `would_create_cycle()` method to prevent cyclic dependencies
- **Ancestor/Descendant Traversal**: `find_ancestors()` and `find_descendants()` methods
- **Depth-based Listing**: `list_at_depth()` to get nodes at specific depth
- **Multiple Parents Support**: True DAG with nodes having multiple parent relationships

### 2. Enhanced Node Metadata
- `created_at` and `updated_at` timestamps on each node
- Node-level metadata HashMap for extensibility

### 3. Quantum-Resistant Cryptography (PQC)
- Optional PQC key pair generation
- Enable/disable via configuration

### 4. Auto-save Functionality
- Configurable auto-save after modifications
- Dirty tracking for unsaved changes

### 5. Vault Configuration System
- Structured `VaultConfig` with:
  - KDF parameters
  - PQC enable/disable
  - Auto-save settings
  - Default password generation length

### 6. Better Error Handling
- More specific error types in errors.rs
- Better error messages for DAG operations

## Implementation Priority
1. Advanced DAG operations (HIGH)
2. Node metadata timestamps (MEDIUM)
3. PQC support (MEDIUM)
4. Configuration system (LOW)
5. Auto-save (LOW)