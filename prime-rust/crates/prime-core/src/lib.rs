//! Core shared structures and protocol definitions for Prime distributed ML

pub mod protocol;
pub mod types;
pub mod error;

pub use error::{Error, Result};
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_setup() {
        // Basic test to ensure the module compiles
        assert_eq!(2 + 2, 4);
    }
}