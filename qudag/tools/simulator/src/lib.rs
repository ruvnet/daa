#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Network simulator for testing and validating QuDAG protocol behavior.

pub mod attacks;
pub mod conditions;
/// Network performance metrics collection and analysis.
pub mod metrics;
/// Network simulation and node management.
pub mod network;
pub mod reports;
/// Test scenario definitions and execution.
pub mod scenarios;
pub mod visualization;

#[cfg(test)]
mod tests {
    mod integration_tests;
    mod metrics_tests;
    mod network_tests;
    mod property_tests;
    mod scenarios_tests;
}
