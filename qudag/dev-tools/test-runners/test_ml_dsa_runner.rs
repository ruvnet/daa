#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! rand = "0.8"
//! colored = "2.0"
//! ```

use std::process::Command;
use std::time::Instant;
use colored::*;

fn main() {
    println!("{}", "ML-DSA Test Suite Runner".bold().blue());
    println!("{}", "=".repeat(50).blue());
    
    let test_categories = vec![
        ("Basic Tests", vec!["test_mldsa_key_generation", "test_mldsa_sign_verify", "test_mldsa_invalid_signature"]),
        ("Comprehensive Tests", vec!["test_ml_dsa_key_generation", "test_ml_dsa_sign_verify", "test_ml_dsa_timing_consistency"]),
        ("Security Tests", vec!["test_ml_dsa_constant_time_verification", "test_ml_dsa_memory_zeroization", "test_ml_dsa_signature_malleability"]),
        ("Property-based Tests", vec!["test_ml_dsa_property_based_correctness", "test_ml_dsa_security_properties"]),
    ];
    
    let mut total_passed = 0;
    let mut total_failed = 0;
    let start_time = Instant::now();
    
    for (category, tests) in test_categories {
        println!("\n{} {}", "Running:".bold(), category.yellow());
        println!("{}", "-".repeat(40).dimmed());
        
        for test_name in tests {
            print!("  {} ", test_name);
            
            let output = Command::new("cargo")
                .args(&["test", "-p", "qudag-crypto", test_name, "--", "--exact", "--nocapture"])
                .output();
                
            match output {
                Ok(result) => {
                    if result.status.success() {
                        println!("{}", "✓ PASSED".green());
                        total_passed += 1;
                    } else {
                        println!("{}", "✗ FAILED".red());
                        total_failed += 1;
                        if !result.stdout.is_empty() {
                            println!("    stdout: {}", String::from_utf8_lossy(&result.stdout).dimmed());
                        }
                        if !result.stderr.is_empty() {
                            println!("    stderr: {}", String::from_utf8_lossy(&result.stderr).dimmed());
                        }
                    }
                }
                Err(e) => {
                    println!("{} ({})", "✗ ERROR".red(), e);
                    total_failed += 1;
                }
            }
        }
    }
    
    let duration = start_time.elapsed();
    
    println!("\n{}", "=".repeat(50).blue());
    println!("{}", "Test Summary".bold().blue());
    println!("  Total tests run: {}", (total_passed + total_failed).to_string().cyan());
    println!("  Passed: {}", total_passed.to_string().green());
    println!("  Failed: {}", total_failed.to_string().red());
    println!("  Duration: {:?}", duration);
    
    if total_failed == 0 {
        println!("\n{}", "All ML-DSA tests passed! ✨".green().bold());
    } else {
        println!("\n{}", "Some tests failed. Please check the output above.".red().bold());
    }
}