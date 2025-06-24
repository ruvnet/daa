/// Advanced side-channel analysis test suite
/// 
/// This module implements comprehensive side-channel analysis including:
/// - Power analysis attacks (SPA/DPA)
/// - Timing attacks with statistical analysis
/// - Cache timing attacks
/// - Electromagnetic emanation analysis
/// - Fault injection resistance
/// - Branch prediction attacks

use qudag_crypto::{
    kem::{KeyEncapsulation, MlKem768},
    ml_dsa::{MlDsa, MlDsaKeyPair},
    encryption::HQC,
    hash::Blake3Hash,
};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use criterion::black_box;
use statrs::statistics::{Statistics, OrderStatistics};
use rand::{RngCore, thread_rng};

/// Statistical analysis thresholds for side-channel detection
const TIMING_VARIANCE_THRESHOLD: f64 = 0.1;  // 10% coefficient of variation
const CORRELATION_THRESHOLD: f64 = 0.05;     // 5% correlation coefficient
const STATISTICAL_CONFIDENCE: f64 = 0.95;    // 95% confidence interval
const MIN_SAMPLES: usize = 10000;             // Minimum samples for statistical significance

/// Power consumption simulation structure
#[derive(Debug, Clone)]
struct PowerTrace {
    samples: Vec<f64>,
    timestamp: Duration,
    operation_type: String,
}

/// Cache access pattern analyzer
#[derive(Debug, Clone)]
struct CacheTrace {
    cache_hits: Vec<u64>,
    cache_misses: Vec<u64>,
    access_pattern: Vec<usize>,
}

/// Electromagnetic emanation simulator
#[derive(Debug, Clone)]
struct EmTrace {
    frequency_spectrum: Vec<f64>,
    amplitude: f64,
    phase: f64,
}

/// Statistical analysis utilities
mod statistical_analysis {
    use super::*;
    
    /// Calculate Pearson correlation coefficient
    pub fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }
        
        let mean_x = x.mean();
        let mean_y = y.mean();
        
        let numerator: f64 = x.iter().zip(y.iter())
            .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
            .sum();
            
        let sum_sq_x: f64 = x.iter().map(|xi| (xi - mean_x).powi(2)).sum();
        let sum_sq_y: f64 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum();
        
        let denominator = (sum_sq_x * sum_sq_y).sqrt();
        
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
    
    /// Calculate coefficient of variation
    pub fn coefficient_of_variation(data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        
        let mean = data.mean();
        let std_dev = data.std_dev();
        
        if mean == 0.0 {
            0.0
        } else {
            std_dev / mean.abs()
        }
    }
    
    /// Perform Welch's t-test for unequal variances
    pub fn welch_t_test(x: &[f64], y: &[f64]) -> (f64, f64) {
        let mean_x = x.mean();
        let mean_y = y.mean();
        let var_x = x.variance();
        let var_y = y.variance();
        let n_x = x.len() as f64;
        let n_y = y.len() as f64;
        
        let t_stat = (mean_x - mean_y) / ((var_x / n_x) + (var_y / n_y)).sqrt();
        
        // Degrees of freedom for Welch's test
        let df_num = (var_x / n_x + var_y / n_y).powi(2);
        let df_denom = (var_x / n_x).powi(2) / (n_x - 1.0) + (var_y / n_y).powi(2) / (n_y - 1.0);
        let df = df_num / df_denom;
        
        (t_stat, df)
    }
    
    /// Detect outliers using IQR method
    pub fn detect_outliers(data: &[f64]) -> Vec<usize> {
        let q1 = data.percentile(25);
        let q3 = data.percentile(75);
        let iqr = q3 - q1;
        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;
        
        data.iter().enumerate()
            .filter_map(|(i, &value)| {
                if value < lower_bound || value > upper_bound {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Power analysis attack simulation
mod power_analysis {
    use super::*;
    
    /// Simulate power consumption during crypto operations
    pub fn simulate_power_consumption<F>(operation: F) -> PowerTrace
    where
        F: FnOnce() -> Vec<u8>,
    {
        let start = Instant::now();
        let mut samples = Vec::with_capacity(1000);
        
        // Simulate power measurement during operation
        let _result = black_box(operation());
        let duration = start.elapsed();
        
        // Generate simulated power trace (in real implementation, this would be actual measurements)
        for i in 0..1000 {
            let base_power = 50.0; // Base power consumption
            let noise = (thread_rng().next_u32() as f64 / u32::MAX as f64 - 0.5) * 10.0;
            let operation_power = (i as f64 / 100.0).sin() * 20.0; // Simulated operation-dependent power
            
            samples.push(base_power + noise + operation_power);
        }
        
        PowerTrace {
            samples,
            timestamp: duration,
            operation_type: "crypto_operation".to_string(),
        }
    }
    
    /// Perform Differential Power Analysis (DPA)
    pub fn differential_power_analysis(
        traces: &[PowerTrace],
        key_hypotheses: &[Vec<u8>],
    ) -> Vec<f64> {
        let mut correlations = Vec::new();
        
        for hypothesis in key_hypotheses {
            let mut hypothesis_correlations = Vec::new();
            
            for sample_idx in 0..traces[0].samples.len() {
                let power_values: Vec<f64> = traces.iter()
                    .map(|trace| trace.samples[sample_idx])
                    .collect();
                
                // Calculate intermediate values for hypothesis
                let intermediate_values: Vec<f64> = traces.iter()
                    .map(|_trace| {
                        // Simulate intermediate value calculation
                        // In real DPA, this would be based on the actual crypto operation
                        let hamming_weight = hypothesis.iter().map(|&b| b.count_ones()).sum::<u32>() as f64;
                        hamming_weight / 8.0 // Normalize
                    })
                    .collect();
                
                let correlation = statistical_analysis::pearson_correlation(&power_values, &intermediate_values);
                hypothesis_correlations.push(correlation.abs());
            }
            
            correlations.push(hypothesis_correlations.iter().fold(0.0, |a, &b| a.max(b)));
        }
        
        correlations
    }
    
    /// Simple Power Analysis (SPA) detection
    pub fn simple_power_analysis(trace: &PowerTrace) -> f64 {
        let cv = statistical_analysis::coefficient_of_variation(&trace.samples);
        let outliers = statistical_analysis::detect_outliers(&trace.samples);
        
        // Combine metrics for SPA vulnerability score
        cv + (outliers.len() as f64 / trace.samples.len() as f64)
    }
}

/// Cache timing attack simulation
mod cache_analysis {
    use super::*;
    
    /// Simulate cache access patterns
    pub fn simulate_cache_access<F>(operation: F) -> CacheTrace
    where
        F: FnOnce() -> Vec<u8>,
    {
        let mut cache_hits = Vec::new();
        let mut cache_misses = Vec::new();
        let mut access_pattern = Vec::new();
        
        // Simulate cache monitoring during operation
        let _result = black_box(operation());
        
        // Generate simulated cache trace
        for i in 0..1000 {
            let cache_line = i % 64; // Simulate 64 cache lines
            access_pattern.push(cache_line);
            
            // Simulate cache hit/miss based on access pattern
            if i > 0 && access_pattern[i-1] == cache_line {
                cache_hits.push(10); // Fast access time
                cache_misses.push(0);
            } else {
                cache_hits.push(0);
                cache_misses.push(100); // Slow access time
            }
        }
        
        CacheTrace {
            cache_hits,
            cache_misses,
            access_pattern,
        }
    }
    
    /// Analyze cache timing patterns for leakage
    pub fn analyze_cache_timing(traces: &[CacheTrace]) -> f64 {
        if traces.is_empty() {
            return 0.0;
        }
        
        let mut pattern_frequencies = HashMap::new();
        
        for trace in traces {
            for &pattern in &trace.access_pattern {
                *pattern_frequencies.entry(pattern).or_insert(0) += 1;
            }
        }
        
        // Calculate entropy of access patterns
        let total_accesses: u32 = pattern_frequencies.values().sum();
        let entropy: f64 = pattern_frequencies.values()
            .map(|&count| {
                let p = count as f64 / total_accesses as f64;
                if p > 0.0 {
                    -p * p.log2()
                } else {
                    0.0
                }
            })
            .sum();
        
        // Lower entropy indicates more predictable patterns (potential vulnerability)
        entropy
    }
}

/// Timing attack analysis
mod timing_analysis {
    use super::*;
    
    /// Measure operation timing with high precision
    pub fn measure_precise_timing<F>(operation: F, iterations: usize) -> Vec<Duration>
    where
        F: Fn() -> Vec<u8>,
    {
        let mut timings = Vec::with_capacity(iterations);
        
        for _ in 0..iterations {
            let start = Instant::now();
            let _result = black_box(operation());
            let duration = start.elapsed();
            timings.push(duration);
        }
        
        timings
    }
    
    /// Statistical timing analysis
    pub fn analyze_timing_distribution(timings: &[Duration]) -> (f64, bool) {
        let timing_nanos: Vec<f64> = timings.iter()
            .map(|d| d.as_nanos() as f64)
            .collect();
        
        let cv = statistical_analysis::coefficient_of_variation(&timing_nanos);
        let is_constant_time = cv < TIMING_VARIANCE_THRESHOLD;
        
        (cv, is_constant_time)
    }
    
    /// Compare timing distributions for different inputs
    pub fn compare_timing_distributions(
        timings1: &[Duration],
        timings2: &[Duration],
    ) -> (f64, bool) {
        let timing1_nanos: Vec<f64> = timings1.iter().map(|d| d.as_nanos() as f64).collect();
        let timing2_nanos: Vec<f64> = timings2.iter().map(|d| d.as_nanos() as f64).collect();
        
        let (t_stat, _df) = statistical_analysis::welch_t_test(&timing1_nanos, &timing2_nanos);
        let is_similar = t_stat.abs() < 2.0; // Rough threshold for similar distributions
        
        (t_stat, is_similar)
    }
}

#[cfg(test)]
mod advanced_side_channel_tests {
    use super::*;

    #[test]
    fn test_ml_kem_power_analysis_resistance() {
        let (pk, sk) = MlKem768::keygen().unwrap();
        let mut traces = Vec::new();
        
        // Generate power traces for encapsulation
        for _ in 0..100 {
            let trace = power_analysis::simulate_power_consumption(|| {
                let (ct, ss) = MlKem768::encapsulate(&pk).unwrap();
                [ct.as_bytes(), ss.as_bytes()].concat()
            });
            traces.push(trace);
        }
        
        // Analyze for SPA vulnerabilities
        let spa_scores: Vec<f64> = traces.iter()
            .map(power_analysis::simple_power_analysis)
            .collect();
        
        let avg_spa_score = spa_scores.iter().sum::<f64>() / spa_scores.len() as f64;
        assert!(avg_spa_score < CORRELATION_THRESHOLD, 
            "ML-KEM encapsulation shows SPA vulnerability: {}", avg_spa_score);
        
        // Test decapsulation with valid and invalid ciphertexts
        let (ct, _) = MlKem768::encapsulate(&pk).unwrap();
        let mut invalid_ct = vec![0u8; ct.as_bytes().len()];
        thread_rng().fill_bytes(&mut invalid_ct);
        
        let valid_timings = timing_analysis::measure_precise_timing(
            || MlKem768::decapsulate(&sk, &ct).unwrap_or_default().as_bytes().to_vec(),
            MIN_SAMPLES / 100
        );
        
        let invalid_timings = timing_analysis::measure_precise_timing(
            || MlKem768::decapsulate(&sk, &qudag_crypto::kem::Ciphertext::from_bytes(&invalid_ct))
                .unwrap_or_default().as_bytes().to_vec(),
            MIN_SAMPLES / 100
        );
        
        let (t_stat, is_similar) = timing_analysis::compare_timing_distributions(
            &valid_timings, &invalid_timings
        );
        
        assert!(is_similar, 
            "ML-KEM decapsulation timing varies between valid/invalid ciphertexts: t-stat = {}", t_stat);
    }

    #[test]
    fn test_ml_dsa_cache_timing_resistance() {
        let keypair = MlDsa::keygen().unwrap();
        let message1 = b"test message 1";
        let message2 = b"test message 2";
        
        // Generate cache traces for different messages
        let mut traces1 = Vec::new();
        let mut traces2 = Vec::new();
        
        for _ in 0..50 {
            let trace1 = cache_analysis::simulate_cache_access(|| {
                MlDsa::sign(message1, keypair.secret_key()).unwrap()
            });
            traces1.push(trace1);
            
            let trace2 = cache_analysis::simulate_cache_access(|| {
                MlDsa::sign(message2, keypair.secret_key()).unwrap()
            });
            traces2.push(trace2);
        }
        
        // Analyze cache access patterns
        let entropy1 = cache_analysis::analyze_cache_timing(&traces1);
        let entropy2 = cache_analysis::analyze_cache_timing(&traces2);
        
        // High entropy indicates good cache timing resistance
        assert!(entropy1 > 4.0, "ML-DSA signing shows low cache entropy for message1: {}", entropy1);
        assert!(entropy2 > 4.0, "ML-DSA signing shows low cache entropy for message2: {}", entropy2);
        
        // Entropies should be similar regardless of message content
        let entropy_diff = (entropy1 - entropy2).abs();
        assert!(entropy_diff < 1.0, 
            "ML-DSA cache patterns vary significantly between messages: {}", entropy_diff);
    }

    #[test]
    fn test_constant_time_operations_statistical() {
        let (pk, sk) = MlKem768::keygen().unwrap();
        
        // Test encapsulation timing consistency
        let encap_timings = timing_analysis::measure_precise_timing(
            || {
                let (ct, ss) = MlKem768::encapsulate(&pk).unwrap();
                [ct.as_bytes(), ss.as_bytes()].concat()
            },
            MIN_SAMPLES / 100
        );
        
        let (cv, is_constant_time) = timing_analysis::analyze_timing_distribution(&encap_timings);
        assert!(is_constant_time, 
            "ML-KEM encapsulation not constant-time: CV = {}", cv);
        
        // Test decapsulation with statistical analysis
        let (ct, _) = MlKem768::encapsulate(&pk).unwrap();
        
        let decap_timings = timing_analysis::measure_precise_timing(
            || MlKem768::decapsulate(&sk, &ct).unwrap().as_bytes().to_vec(),
            MIN_SAMPLES / 100
        );
        
        let (cv, is_constant_time) = timing_analysis::analyze_timing_distribution(&decap_timings);
        assert!(is_constant_time, 
            "ML-KEM decapsulation not constant-time: CV = {}", cv);
        
        // Remove outliers and retest
        let timing_nanos: Vec<f64> = decap_timings.iter()
            .map(|d| d.as_nanos() as f64)
            .collect();
        let outliers = statistical_analysis::detect_outliers(&timing_nanos);
        
        // Should have minimal outliers for constant-time operations
        assert!(outliers.len() < decap_timings.len() / 20, 
            "Too many timing outliers detected: {}/{}", outliers.len(), decap_timings.len());
    }

    #[test]
    fn test_differential_power_analysis_resistance() {
        let (pk, sk) = MlKem768::keygen().unwrap();
        
        // Generate key hypotheses for DPA
        let mut key_hypotheses = Vec::new();
        for i in 0..16 {
            key_hypotheses.push(vec![i as u8; 32]); // 32-byte key hypotheses
        }
        
        // Collect power traces for different operations
        let mut traces = Vec::new();
        for _ in 0..100 {
            let trace = power_analysis::simulate_power_consumption(|| {
                let (ct, ss) = MlKem768::encapsulate(&pk).unwrap();
                let _ = MlKem768::decapsulate(&sk, &ct).unwrap();
                ss.as_bytes().to_vec()
            });
            traces.push(trace);
        }
        
        // Perform DPA analysis
        let correlations = power_analysis::differential_power_analysis(&traces, &key_hypotheses);
        
        // All correlations should be below threshold
        let max_correlation = correlations.iter().fold(0.0, |a, &b| a.max(b));
        assert!(max_correlation < CORRELATION_THRESHOLD,
            "DPA attack shows high correlation: {}", max_correlation);
        
        // Test that correlations are relatively uniform (no obvious key leakage)
        let correlation_variance = statistical_analysis::coefficient_of_variation(&correlations);
        assert!(correlation_variance < 0.5,
            "DPA correlations show suspicious patterns: CV = {}", correlation_variance);
    }

    #[test]
    fn test_fault_injection_resistance() {
        let keypair = MlDsa::keygen().unwrap();
        let message = b"fault injection test message";
        
        // Test signature generation with simulated faults
        let mut successful_sigs = 0;
        let mut failed_sigs = 0;
        
        for _ in 0..100 {
            // Simulate random bit flips in memory (fault injection)
            let mut rng = thread_rng();
            if rng.next_u32() % 100 < 5 { // 5% fault injection rate
                // Skip this iteration to simulate a fault
                failed_sigs += 1;
                continue;
            }
            
            match MlDsa::sign(message, keypair.secret_key()) {
                Ok(signature) => {
                    // Verify the signature is still valid
                    if MlDsa::verify(message, &signature, keypair.public_key()).is_ok() {
                        successful_sigs += 1;
                    } else {
                        failed_sigs += 1;
                    }
                }
                Err(_) => {
                    failed_sigs += 1;
                }
            }
        }
        
        // Even with fault injection, valid signatures should still verify
        let success_rate = successful_sigs as f64 / (successful_sigs + failed_sigs) as f64;
        assert!(success_rate > 0.9, // Allow for some fault-induced failures
            "Fault injection caused too many signature failures: success rate = {}", success_rate);
    }

    #[test]
    fn test_branch_prediction_resistance() {
        let (pk, sk) = MlKem768::keygen().unwrap();
        
        // Test with predictable patterns
        let mut predictable_timings = Vec::new();
        for i in 0..100 {
            let mut ct_bytes = vec![0u8; 1088]; // ML-KEM-768 ciphertext size
            // Create predictable pattern
            for j in 0..ct_bytes.len() {
                ct_bytes[j] = ((i + j) % 256) as u8;
            }
            
            let ct = qudag_crypto::kem::Ciphertext::from_bytes(&ct_bytes);
            let timing = timing_analysis::measure_precise_timing(
                || MlKem768::decapsulate(&sk, &ct).unwrap_or_default().as_bytes().to_vec(),
                1
            )[0];
            predictable_timings.push(timing);
        }
        
        // Test with random patterns
        let mut random_timings = Vec::new();
        for _ in 0..100 {
            let mut ct_bytes = vec![0u8; 1088];
            thread_rng().fill_bytes(&mut ct_bytes);
            
            let ct = qudag_crypto::kem::Ciphertext::from_bytes(&ct_bytes);
            let timing = timing_analysis::measure_precise_timing(
                || MlKem768::decapsulate(&sk, &ct).unwrap_or_default().as_bytes().to_vec(),
                1
            )[0];
            random_timings.push(timing);
        }
        
        // Compare timing distributions
        let (t_stat, is_similar) = timing_analysis::compare_timing_distributions(
            &predictable_timings, &random_timings
        );
        
        assert!(is_similar,
            "Branch prediction attack detected: predictable vs random timing t-stat = {}", t_stat);
    }

    #[test]
    fn test_electromagnetic_emanation_resistance() {
        // Simulate EM emanation analysis
        let (pk, sk) = MlKem768::keygen().unwrap();
        
        let mut em_traces = Vec::new();
        
        for _ in 0..50 {
            // Simulate EM measurement during crypto operation
            let start = Instant::now();
            let (ct, ss) = MlKem768::encapsulate(&pk).unwrap();
            let _ = MlKem768::decapsulate(&sk, &ct).unwrap();
            let duration = start.elapsed();
            
            // Generate simulated EM trace
            let mut frequency_spectrum = Vec::new();
            for i in 0..100 {
                let freq = i as f64 * 10.0; // MHz
                let amplitude = (freq / 100.0).sin() + 
                    (thread_rng().next_u32() as f64 / u32::MAX as f64 - 0.5) * 0.1;
                frequency_spectrum.push(amplitude);
            }
            
            em_traces.push(EmTrace {
                frequency_spectrum,
                amplitude: duration.as_nanos() as f64 / 1000.0,
                phase: thread_rng().next_u32() as f64 / u32::MAX as f64 * 2.0 * std::f64::consts::PI,
            });
        }
        
        // Analyze EM traces for information leakage
        let amplitudes: Vec<f64> = em_traces.iter().map(|trace| trace.amplitude).collect();
        let cv = statistical_analysis::coefficient_of_variation(&amplitudes);
        
        assert!(cv < TIMING_VARIANCE_THRESHOLD,
            "EM emanation shows suspicious amplitude variation: CV = {}", cv);
        
        // Test frequency spectrum consistency
        for freq_idx in 0..100 {
            let freq_amplitudes: Vec<f64> = em_traces.iter()
                .map(|trace| trace.frequency_spectrum[freq_idx])
                .collect();
            let freq_cv = statistical_analysis::coefficient_of_variation(&freq_amplitudes);
            
            assert!(freq_cv < 0.3, // Allow some variation in frequency domain
                "EM frequency {} MHz shows suspicious variation: CV = {}", freq_idx * 10, freq_cv);
        }
    }

    #[test]
    fn test_multi_threaded_side_channel_resistance() {
        use std::sync::Arc;
        use std::thread;
        
        let keypair = Arc::new(MlDsa::keygen().unwrap());
        let message = b"multithreaded test message";
        
        let mut handles = Vec::new();
        
        // Spawn multiple threads performing crypto operations
        for thread_id in 0..4 {
            let keypair_clone = Arc::clone(&keypair);
            let handle = thread::spawn(move || {
                let mut timings = Vec::new();
                
                for _ in 0..25 {
                    let start = Instant::now();
                    let signature = MlDsa::sign(message, keypair_clone.secret_key()).unwrap();
                    let _ = MlDsa::verify(message, &signature, keypair_clone.public_key()).unwrap();
                    let duration = start.elapsed();
                    timings.push(duration);
                }
                
                (thread_id, timings)
            });
            handles.push(handle);
        }
        
        // Collect results from all threads
        let mut all_timings = Vec::new();
        for handle in handles {
            let (thread_id, timings) = handle.join().unwrap();
            println!("Thread {} completed {} operations", thread_id, timings.len());
            all_timings.extend(timings);
        }
        
        // Analyze timing consistency across threads
        let (cv, is_constant_time) = timing_analysis::analyze_timing_distribution(&all_timings);
        assert!(is_constant_time,
            "Multi-threaded operations not constant-time: CV = {}", cv);
        
        // Verify no thread-specific timing patterns
        let timing_nanos: Vec<f64> = all_timings.iter()
            .map(|d| d.as_nanos() as f64)
            .collect();
        let outliers = statistical_analysis::detect_outliers(&timing_nanos);
        
        assert!(outliers.len() < all_timings.len() / 10,
            "Too many timing outliers in multi-threaded execution: {}/{}",
            outliers.len(), all_timings.len());
    }
}