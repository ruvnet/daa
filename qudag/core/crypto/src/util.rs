use constant_time_eq::constant_time_eq;
use subtle::ConstantTimeEq;

/// Constant-time comparison of two byte slices
#[inline]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    constant_time_eq(a, b)
}

/// Constant-time conditional copy
#[inline]
pub fn ct_select(a: &[u8], b: &[u8], choice: u8) -> Vec<u8> {
    debug_assert_eq!(a.len(), b.len());
    let choice_u8 = choice & 1;
    
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| {
            let z = (x & !choice_u8) | (y & choice_u8);
            z
        })
        .collect()
}