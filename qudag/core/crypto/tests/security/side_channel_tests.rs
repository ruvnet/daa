use qudag_crypto::{kem::MLKem, signatures::MLDsa, encryption::HQC};
use test_utils::power_analysis::*;
use std::time::{Duration, Instant};

/// Side-channel analysis test suite for cryptographic operations
#[cfg(test)]
mod side_channel_tests {
    use super::*;

    const TRACE_SAMPLES: usize = 10000;
    const POWER_THRESHOLD: f64 = 0.05; // Maximum allowed correlation coefficient

    /// Helper function to collect power traces during operation
    fn collect_power_trace<F>(operation: F) -> Vec<f64>
    where
        F: Fn() -> Vec<u8>,
    {
        let mut traces = Vec::with_capacity(TRACE_SAMPLES);
        
        for _ in 0..TRACE_SAMPLES {
            let start_power = measure_power_usage();
            let start = Instant::now();
            
            let result = operation();
            black_box(result);
            
            let duration = start.elapsed();
            let end_power = measure_power_usage();
            
            // Calculate normalized power consumption
            let power_usage = (end_power - start_power) / duration.as_micros() as f64;
            traces.push(power_usage);
        }
        traces
    }

    #[test]
    fn test_mlkem_power_analysis() {
        let (pk, sk) = MLKem::keygen();
        
        // Collect traces for encapsulation
        let enc_traces = collect_power_trace(|| {
            let (ct, ss) = MLKem::encapsulate(&pk).unwrap();
            [ct, ss].concat()
        });
        
        // Collect traces for decapsulation
        let (ct, _) = MLKem::encapsulate(&pk).unwrap();
        let dec_traces = collect_power_trace(|| {
            MLKem::decapsulate(&ct, &sk).unwrap()
        });
        
        // Test for correlations that could leak key information
        let enc_correlation = analyze_power_correlation(&enc_traces);
        let dec_correlation = analyze_power_correlation(&dec_traces);
        
        assert!(enc_correlation < POWER_THRESHOLD,
            "ML-KEM encapsulation shows suspicious power correlation");
        assert!(dec_correlation < POWER_THRESHOLD,
            "ML-KEM decapsulation shows suspicious power correlation");
    }

    #[test]
    fn test_mldsa_power_analysis() {
        let message = b"test message";
        let (pk, sk) = MLDsa::keygen();
        
        // Collect traces for signing
        let sign_traces = collect_power_trace(|| {
            MLDsa::sign(message, &sk)
        });
        
        // Collect traces for verification
        let signature = MLDsa::sign(message, &sk);
        let verify_traces = collect_power_trace(|| {
            let valid = MLDsa::verify(message, &signature, &pk).is_ok();
            vec![valid as u8]
        });
        
        let sign_correlation = analyze_power_correlation(&sign_traces);
        let verify_correlation = analyze_power_correlation(&verify_traces);
        
        assert!(sign_correlation < POWER_THRESHOLD,
            "ML-DSA signing shows suspicious power correlation");
        assert!(verify_correlation < POWER_THRESHOLD,
            "ML-DSA verification shows suspicious power correlation");
    }

    #[test]
    fn test_hqc_power_analysis() {
        let message = b"test message";
        let (pk, sk) = HQC::keygen();
        
        // Collect traces for encryption
        let enc_traces = collect_power_trace(|| {
            HQC::encrypt(message, &pk).unwrap()
        });
        
        // Collect traces for decryption
        let ct = HQC::encrypt(message, &pk).unwrap();
        let dec_traces = collect_power_trace(|| {
            HQC::decrypt(&ct, &sk).unwrap()
        });
        
        let enc_correlation = analyze_power_correlation(&enc_traces);
        let dec_correlation = analyze_power_correlation(&dec_traces);
        
        assert!(enc_correlation < POWER_THRESHOLD,
            "HQC encryption shows suspicious power correlation");
        assert!(dec_correlation < POWER_THRESHOLD,
            "HQC decryption shows suspicious power correlation");
    }

    #[test]
    fn test_cache_timing_analysis() {
        let message = b"test message";
        let (pk, sk) = MLKem::keygen();
        
        // Test for cache timing attacks by measuring access patterns
        let cache_traces = collect_cache_traces(|| {
            let (ct, _) = MLKem::encapsulate(&pk).unwrap();
            let _ = MLKem::decapsulate(&ct, &sk).unwrap();
        });
        
        // Analyze cache access patterns for potential leaks
        let cache_correlation = analyze_cache_correlation(&cache_traces);
        assert!(cache_correlation < POWER_THRESHOLD,
            "Suspicious cache access patterns detected");
    }

    #[test]
    fn test_branch_timing_analysis() {
        let message = b"test message";
        let (pk, sk) = MLDsa::keygen();
        let signature = MLDsa::sign(message, &sk);
        
        // Test for timing variations in branches
        let branch_traces = collect_branch_traces(|| {
            MLDsa::verify(message, &signature, &pk).is_ok()
        });
        
        let branch_correlation = analyze_branch_correlation(&branch_traces);
        assert!(branch_correlation < POWER_THRESHOLD,
            "Suspicious branch timing variations detected");
    }
}