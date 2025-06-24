# Onion.rs Error Fixes

## Errors Fixed

### 1. Removed non-existent method call
**Error**: `no method named 'generate_symmetric_key'`
**Line**: 297
**Fix**: Removed the call to `self.generate_symmetric_key()?` as the symmetric key is derived from the ML-KEM shared secret in the subsequent lines.

### 2. Fixed Vec<u8> method call
**Error**: `no method named 'to_bytes' found for struct 'Vec<u8>'`
**Line**: 322
**Fix**: Changed `route[route.len() - i].to_bytes().to_vec()` to `route[route.len() - i].clone()` since `route` already contains `Vec<u8>` elements.

### 3. Fixed HKDF expand call
**Error**: Type mismatch in `prk.expand()` call
**Line**: 219
**Fix**: Changed the info parameter to use proper slice references: `&info[..]` instead of `&info`.

## Summary

All errors in `core/network/src/onion.rs` have been successfully resolved. The file now compiles without errors. The fixes maintain the intended functionality while correcting the method calls to match the actual API provided by the crypto and standard libraries.