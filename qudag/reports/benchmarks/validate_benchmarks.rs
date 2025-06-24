#!/usr/bin/env rust-script

//! Simple benchmark validation tool
//! 
//! This script validates that benchmark files are syntactically correct
//! and can be compiled without running the actual benchmarks.

use std::process::Command;
use std::fs;
use std::path::Path;

fn main() {
    println!("=== Benchmark Validation Tool ===\n");
    
    let modules = vec!["crypto", "dag", "network", "protocol"];
    let mut all_valid = true;
    
    for module in modules {
        println!("Validating {} benchmarks...", module);
        
        let bench_dir = format!("core/{}/benches", module);
        if !Path::new(&bench_dir).exists() {
            println!("  ❌ No benchmark directory found");
            all_valid = false;
            continue;
        }
        
        // Get benchmark files
        let bench_files: Vec<_> = fs::read_dir(&bench_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "rs" {
                    Some(path.file_stem()?.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect();
            
        if bench_files.is_empty() {
            println!("  ❌ No benchmark files found");
            all_valid = false;
            continue;
        }
        
        println!("  Found {} benchmark files", bench_files.len());
        
        for bench_file in &bench_files {
            print!("    Checking {}... ", bench_file);
            
            // Try to check compilation syntax
            let output = Command::new("cargo")
                .args(&["check", "--bench", bench_file, "-p", &format!("qudag-{}", module)])
                .output();
                
            match output {
                Ok(result) if result.status.success() => {
                    println!("✅ OK");
                }
                Ok(result) => {
                    println!("❌ Compilation Error");
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    if !stderr.is_empty() {
                        println!("      Error: {}", stderr.lines().next().unwrap_or("Unknown error"));
                    }
                    all_valid = false;
                }
                Err(e) => {
                    println!("❌ Failed to run check: {}", e);
                    all_valid = false;
                }
            }
        }
        println!();
    }
    
    // Summary
    if all_valid {
        println!("🎉 All benchmarks are syntactically valid!");
    } else {
        println!("⚠️  Some benchmarks have issues that need to be fixed.");
        std::process::exit(1);
    }
}