/// Comprehensive timing attack resistance test suite
/// 
/// This module implements detailed timing attack analysis including:
/// - Constant-time verification for all crypto operations
/// - Statistical timing analysis with multiple metrics
/// - Remote timing attack simulation
/// - Micro-architectural timing attacks
/// - Network timing attack resistance

use qudag_crypto::{
    kem::{KeyEncapsulation, MlKem768},
    ml_dsa::{MlDsa, MlDsaKeyPair},
    encryption::HQC,
    hash::Blake3Hash,
};
use std::time::{Duration, Instant, SystemTime};
use std::collections::{HashMap, VecDeque};
use criterion::black_box;
use rand::{RngCore, thread_rng, Rng};
use std::sync::{Arc, Mutex};
use std::thread;

/// Timing measurement precision and thresholds
const HIGH_PRECISION_SAMPLES: usize = 50000;
const STATISTICAL_SAMPLES: usize = 10000;
const TIMING_VARIANCE_THRESHOLD: f64 = 0.05; // 5% coefficient of variation
const TIMING_DIFFERENCE_THRESHOLD: Duration = Duration::from_nanos(1000); // 1μs threshold
const CONFIDENCE_LEVEL: f64 = 0.95;

/// High-precision timing measurement structure
#[derive(Debug, Clone)]
struct PrecisionTiming {
    measurements: Vec<Duration>,
    operation_type: String,
    input_characteristics: HashMap<String, String>,
}

/// Timing attack scenario configuration
#[derive(Debug, Clone)]
struct TimingAttackScenario {
    name: String,
    description: String,
    attack_vector: AttackVector,
    expected_constant_time: bool,
}

#[derive(Debug, Clone)]
enum AttackVector {
    LocalTiming,
    RemoteTiming,
    CacheTiming,
    BranchTiming,
    MemoryTiming,
    NetworkTiming,
}

/// Statistical timing analysis utilities
mod timing_statistics {
    use super::*;
    use statrs::statistics::{Statistics, OrderStatistics};
    
    /// Comprehensive timing statistics
    #[derive(Debug, Clone)]
    pub struct TimingStats {
        pub mean: Duration,
        pub median: Duration,
        pub std_dev: Duration,
        pub min: Duration,
        pub max: Duration,
        pub percentile_95: Duration,
        pub percentile_99: Duration,
        pub coefficient_of_variation: f64,
        pub outlier_count: usize,
        pub sample_count: usize,
    }
    
    pub fn calculate_timing_stats(timings: &[Duration]) -> TimingStats {
        if timings.is_empty() {
            return TimingStats {
                mean: Duration::ZERO,
                median: Duration::ZERO,
                std_dev: Duration::ZERO,
                min: Duration::ZERO,
                max: Duration::ZERO,
                percentile_95: Duration::ZERO,
                percentile_99: Duration::ZERO,
                coefficient_of_variation: 0.0,
                outlier_count: 0,
                sample_count: 0,
            };
        }
        
        let nanos: Vec<f64> = timings.iter().map(|d| d.as_nanos() as f64).collect();
        
        let mean_nanos = nanos.mean();
        let median_nanos = nanos.median();
        let std_dev_nanos = nanos.std_dev();
        let min_nanos = nanos.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_nanos = nanos.iter().fold(0.0, |a, &b| a.max(b));
        let percentile_95_nanos = nanos.percentile(95);
        let percentile_99_nanos = nanos.percentile(99);
        
        let cv = if mean_nanos > 0.0 { std_dev_nanos / mean_nanos } else { 0.0 };
        
        // Count outliers using IQR method
        let q1 = nanos.percentile(25);
        let q3 = nanos.percentile(75);
        let iqr = q3 - q1;
        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;
        
        let outlier_count = nanos.iter()
            .filter(|&&value| value < lower_bound || value > upper_bound)
            .count();
        
        TimingStats {
            mean: Duration::from_nanos(mean_nanos as u64),
            median: Duration::from_nanos(median_nanos as u64),
            std_dev: Duration::from_nanos(std_dev_nanos as u64),
            min: Duration::from_nanos(min_nanos as u64),
            max: Duration::from_nanos(max_nanos as u64),
            percentile_95: Duration::from_nanos(percentile_95_nanos as u64),
            percentile_99: Duration::from_nanos(percentile_99_nanos as u64),
            coefficient_of_variation: cv,
            outlier_count,
            sample_count: timings.len(),
        }
    }
    
    /// Compare two timing distributions using statistical tests
    pub fn compare_timing_distributions(
        timings1: &[Duration],
        timings2: &[Duration],
    ) -> (f64, bool, String) {
        let nanos1: Vec<f64> = timings1.iter().map(|d| d.as_nanos() as f64).collect();
        let nanos2: Vec<f64> = timings2.iter().map(|d| d.as_nanos() as f64).collect();
        
        // Welch's t-test for unequal variances
        let mean1 = nanos1.mean();
        let mean2 = nanos2.mean();
        let var1 = nanos1.variance();
        let var2 = nanos2.variance();
        let n1 = nanos1.len() as f64;
        let n2 = nanos2.len() as f64;
        
        let t_statistic = (mean1 - mean2) / ((var1 / n1) + (var2 / n2)).sqrt();
        
        // Mann-Whitney U test (non-parametric)
        let combined: Vec<(f64, usize)> = nanos1.iter().map(|&x| (x, 0))
            .chain(nanos2.iter().map(|&x| (x, 1)))
            .collect();
        
        let mut sorted_combined = combined;
        sorted_combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        
        let mut u1 = 0.0;
        for (i, &(_, group)) in sorted_combined.iter().enumerate() {
            if group == 0 {
                u1 += (i + 1) as f64;
            }
        }
        u1 -= n1 * (n1 + 1.0) / 2.0;
        
        let u2 = n1 * n2 - u1;
        let u_statistic = u1.min(u2);
        let u_critical = n1 * n2 / 2.0 - 1.96 * (n1 * n2 * (n1 + n2 + 1.0) / 12.0).sqrt();
        
        let distributions_similar = t_statistic.abs() < 2.0 && u_statistic > u_critical;
        
        let analysis = format!(
            "t-statistic: {:.4}, U-statistic: {:.2}, U-critical: {:.2}",
            t_statistic, u_statistic, u_critical
        );
        
        (t_statistic, distributions_similar, analysis)
    }
    
    /// Detect timing anomalies using multiple detection methods
    pub fn detect_timing_anomalies(timings: &[Duration]) -> Vec<(usize, String)> {
        let mut anomalies = Vec::new();
        let nanos: Vec<f64> = timings.iter().map(|d| d.as_nanos() as f64).collect();
        
        // Method 1: IQR-based outlier detection
        let q1 = nanos.percentile(25);
        let q3 = nanos.percentile(75);
        let iqr = q3 - q1;
        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;
        
        for (i, &value) in nanos.iter().enumerate() {
            if value < lower_bound {
                anomalies.push((i, format!("IQR outlier (low): {:.2}ns", value)));
            } else if value > upper_bound {
                anomalies.push((i, format!("IQR outlier (high): {:.2}ns", value)));
            }
        }
        
        // Method 2: Z-score based detection
        let mean = nanos.mean();
        let std_dev = nanos.std_dev();
        
        for (i, &value) in nanos.iter().enumerate() {
            let z_score = (value - mean) / std_dev;
            if z_score.abs() > 3.0 {
                anomalies.push((i, format!("Z-score outlier: {:.2} (z={:.2})", value, z_score)));
            }
        }
        
        // Method 3: Local outlier factor (simplified)
        let k = 20.min(nanos.len() / 10); // k-nearest neighbors
        for (i, &value) in nanos.iter().enumerate() {
            let mut distances: Vec<f64> = nanos.iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, &other)| (value - other).abs())
                .collect();
            distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            if distances.len() >= k {
                let local_density = distances[..k].iter().sum::<f64>() / k as f64;
                if local_density > std_dev * 2.0 {
                    anomalies.push((i, format!("LOF outlier: {:.2}ns (density={:.2})", value, local_density)));
                }
            }
        }
        
        // Remove duplicates
        anomalies.sort_by_key(|(i, _)| *i);
        anomalies.dedup_by_key(|(i, _)| *i);
        
        anomalies
    }
}

/// High-precision timing measurement utilities
mod precision_timing {
    use super::*;
    
    /// Measure operation timing with CPU cycle precision where available
    pub fn measure_cpu_cycles<F, R>(operation: F) -> (R, u64)
    where
        F: FnOnce() -> R,
    {
        // Use RDTSC on x86_64 if available, fallback to high-precision timer
        #[cfg(target_arch = "x86_64")]
        {
            unsafe {
                let start = std::arch::x86_64::_rdtsc();
                let result = black_box(operation());
                let end = std::arch::x86_64::_rdtsc();
                (result, end.wrapping_sub(start))
            }
        }
        
        #[cfg(not(target_arch = "x86_64"))]
        {
            let start = Instant::now();
            let result = black_box(operation());
            let cycles = start.elapsed().as_nanos() as u64; // Approximate cycles
            (result, cycles)
        }
    }
    
    /// Measure with multiple timing sources for validation
    pub fn measure_multi_source<F, R>(operation: F) -> (R, Duration, u64)
    where
        F: FnOnce() -> R,
    {
        let system_start = SystemTime::now();
        let instant_start = Instant::now();
        
        let (result, cycles) = measure_cpu_cycles(operation);
        
        let instant_duration = instant_start.elapsed();
        let system_duration = system_start.elapsed().unwrap_or(Duration::ZERO);
        
        // Use the more precise measurement
        let duration = if instant_duration < system_duration {
            instant_duration
        } else {
            system_duration
        };
        
        (result, duration, cycles)
    }
    
    /// Warm up CPU and stabilize timing measurements
    pub fn warmup_cpu() {
        for _ in 0..1000 {
            let _ = black_box(thread_rng().next_u64());
        }
        
        // Perform some crypto operations to warm up the implementation
        let (pk, sk) = MlKem768::keygen().unwrap();
        for _ in 0..10 {
            let (ct, _ss) = MlKem768::encapsulate(&pk).unwrap();
            let _ = MlKem768::decapsulate(&sk, &ct).unwrap();
        }
    }
}

/// Remote timing attack simulation
mod remote_timing {
    use super::*;
    use std::sync::mpsc;
    
    /// Simulate network latency and jitter
    pub fn simulate_network_delay() -> Duration {
        let base_latency = Duration::from_millis(10); // 10ms base latency
        let jitter = Duration::from_micros(thread_rng().gen_range(0..5000)); // Up to 5ms jitter
        base_latency + jitter
    }
    
    /// Simulate remote timing attack scenario
    pub fn simulate_remote_attack<F>(
        operation: F,
        samples: usize,
    ) -> Vec<Duration>
    where
        F: Fn() -> Vec<u8> + Send + 'static + Clone,
    {
        let (sender, receiver) = mpsc::channel();
        let operation = Arc::new(operation);
        
        // Spawn worker thread to simulate remote execution
        let op_clone = Arc::clone(&operation);
        let worker_sender = sender.clone();
        thread::spawn(move || {
            for i in 0..samples {
                let network_delay = simulate_network_delay();
                
                let start = Instant::now();
                thread::sleep(network_delay); // Simulate network delay to server
                
                let _ = black_box(op_clone());
                
                thread::sleep(network_delay); // Simulate network delay from server
                let total_time = start.elapsed();
                
                worker_sender.send((i, total_time)).unwrap();
            }
        });
        
        // Collect timing measurements
        let mut timings = Vec::new();
        for _ in 0..samples {
            let (_i, timing) = receiver.recv().unwrap();
            timings.push(timing);
        }
        
        timings
    }
}

#[cfg(test)]
mod timing_attack_tests {
    use super::*;

    #[test]
    fn test_ml_kem_constant_time_keygen() {
        precision_timing::warmup_cpu();
        
        let mut timings = Vec::new();
        
        for _ in 0..STATISTICAL_SAMPLES {
            let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                MlKem768::keygen().unwrap()
            });
            timings.push(duration);
        }
        
        let stats = timing_statistics::calculate_timing_stats(&timings);
        
        println!("ML-KEM Key Generation Timing Stats:");
        println!("  Mean: {:?}", stats.mean);
        println!("  Std Dev: {:?}", stats.std_dev);
        println!("  CV: {:.6}", stats.coefficient_of_variation);
        println!("  Outliers: {}/{}", stats.outlier_count, stats.sample_count);
        
        assert!(stats.coefficient_of_variation < TIMING_VARIANCE_THRESHOLD,
            "ML-KEM key generation not constant-time: CV = {:.6}", stats.coefficient_of_variation);
        
        let anomalies = timing_statistics::detect_timing_anomalies(&timings);
        assert!(anomalies.len() < stats.sample_count / 20,
            "Too many timing anomalies in key generation: {}", anomalies.len());
    }

    #[test]
    fn test_ml_kem_constant_time_encapsulation() {
        precision_timing::warmup_cpu();
        
        // Generate multiple keypairs to test timing consistency across different keys
        let mut all_timings = Vec::new();
        
        for _ in 0..10 {
            let (pk, _sk) = MlKem768::keygen().unwrap();
            let mut timings = Vec::new();
            
            for _ in 0..1000 {
                let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                    MlKem768::encapsulate(&pk).unwrap()
                });
                timings.push(duration);
            }
            
            all_timings.extend(timings);
        }
        
        let stats = timing_statistics::calculate_timing_stats(&all_timings);
        
        assert!(stats.coefficient_of_variation < TIMING_VARIANCE_THRESHOLD,
            "ML-KEM encapsulation not constant-time: CV = {:.6}", stats.coefficient_of_variation);
        
        // Test that timing doesn't depend on public key content
        let key_specific_timings = all_timings.chunks(1000).collect::<Vec<_>>();
        for i in 1..key_specific_timings.len() {
            let (t_stat, similar, analysis) = timing_statistics::compare_timing_distributions(
                key_specific_timings[0],
                key_specific_timings[i],
            );
            
            assert!(similar,
                "Encapsulation timing varies between keys: {}", analysis);
        }
    }

    #[test]
    fn test_ml_kem_constant_time_decapsulation() {
        precision_timing::warmup_cpu();
        
        let (pk, sk) = MlKem768::keygen().unwrap();
        
        // Test with valid ciphertext
        let (ct, _ss) = MlKem768::encapsulate(&pk).unwrap();
        let mut valid_timings = Vec::new();
        
        for _ in 0..STATISTICAL_SAMPLES / 2 {
            let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                MlKem768::decapsulate(&sk, &ct).unwrap()
            });
            valid_timings.push(duration);
        }
        
        // Test with invalid ciphertext
        let mut invalid_timings = Vec::new();
        for _ in 0..STATISTICAL_SAMPLES / 2 {
            let mut invalid_ct_bytes = vec![0u8; 1088]; // ML-KEM-768 ciphertext size
            thread_rng().fill_bytes(&mut invalid_ct_bytes);
            let invalid_ct = qudag_crypto::kem::Ciphertext::from_bytes(&invalid_ct_bytes);
            
            let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                let _ = MlKem768::decapsulate(&sk, &invalid_ct); // May fail, but timing should be constant
            });
            invalid_timings.push(duration);
        }
        
        // Analyze timing distributions
        let valid_stats = timing_statistics::calculate_timing_stats(&valid_timings);
        let invalid_stats = timing_statistics::calculate_timing_stats(&invalid_timings);
        
        println!("ML-KEM Decapsulation Timing:");
        println!("  Valid CT - CV: {:.6}, Mean: {:?}", valid_stats.coefficient_of_variation, valid_stats.mean);
        println!("  Invalid CT - CV: {:.6}, Mean: {:?}", invalid_stats.coefficient_of_variation, invalid_stats.mean);
        
        // Both should be constant-time
        assert!(valid_stats.coefficient_of_variation < TIMING_VARIANCE_THRESHOLD,
            "Valid decapsulation not constant-time: CV = {:.6}", valid_stats.coefficient_of_variation);
        assert!(invalid_stats.coefficient_of_variation < TIMING_VARIANCE_THRESHOLD,
            "Invalid decapsulation not constant-time: CV = {:.6}", invalid_stats.coefficient_of_variation);
        
        // Timing should be similar regardless of ciphertext validity
        let (t_stat, similar, analysis) = timing_statistics::compare_timing_distributions(
            &valid_timings, &invalid_timings
        );
        
        assert!(similar,
            "Decapsulation timing differs between valid/invalid ciphertext: {}", analysis);
    }

    #[test]
    fn test_ml_dsa_constant_time_signing() {
        precision_timing::warmup_cpu();
        
        let keypair = MlDsa::keygen().unwrap();
        
        // Test with messages of different lengths
        let test_messages = vec![
            vec![],                    // Empty message
            vec![0x42],               // Single byte
            vec![0x42; 16],           // 16 bytes
            vec![0x42; 64],           // 64 bytes
            vec![0x42; 256],          // 256 bytes
            vec![0x42; 1024],         // 1KB message
        ];
        
        for (msg_len, message) in test_messages.iter().enumerate() {
            let mut timings = Vec::new();
            
            // Test with different message content but same length
            for _ in 0..1000 {
                let mut test_msg = message.clone();
                if !test_msg.is_empty() {
                    // Randomize content while keeping length constant
                    for byte in test_msg.iter_mut() {
                        *byte = thread_rng().gen();
                    }
                }
                
                let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                    MlDsa::sign(&test_msg, keypair.secret_key()).unwrap()
                });
                timings.push(duration);
            }
            
            let stats = timing_statistics::calculate_timing_stats(&timings);
            
            println!("ML-DSA Signing ({}B): CV = {:.6}, Mean = {:?}", 
                message.len(), stats.coefficient_of_variation, stats.mean);
            
            assert!(stats.coefficient_of_variation < TIMING_VARIANCE_THRESHOLD * 2.0, // Allow more variance for signing
                "ML-DSA signing not constant-time for {}B messages: CV = {:.6}", 
                message.len(), stats.coefficient_of_variation);
        }
    }

    #[test]
    fn test_ml_dsa_constant_time_verification() {
        precision_timing::warmup_cpu();
        
        let keypair = MlDsa::keygen().unwrap();
        let message = b"test message for verification timing";
        let valid_signature = MlDsa::sign(message, keypair.secret_key()).unwrap();
        
        // Test valid signature verification
        let mut valid_timings = Vec::new();
        for _ in 0..STATISTICAL_SAMPLES / 2 {
            let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                MlDsa::verify(message, &valid_signature, keypair.public_key()).is_ok()
            });
            valid_timings.push(duration);
        }
        
        // Test invalid signature verification
        let mut invalid_timings = Vec::new();
        for _ in 0..STATISTICAL_SAMPLES / 2 {
            let mut invalid_sig = valid_signature.clone();
            // Introduce random errors in signature
            let error_pos = thread_rng().gen_range(0..invalid_sig.len());
            invalid_sig[error_pos] ^= 1;
            
            let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                MlDsa::verify(message, &invalid_sig, keypair.public_key()).is_ok()
            });
            invalid_timings.push(duration);
        }
        
        let valid_stats = timing_statistics::calculate_timing_stats(&valid_timings);
        let invalid_stats = timing_statistics::calculate_timing_stats(&invalid_timings);
        
        // Both should be constant-time
        assert!(valid_stats.coefficient_of_variation < TIMING_VARIANCE_THRESHOLD,
            "Valid signature verification not constant-time: CV = {:.6}", valid_stats.coefficient_of_variation);
        assert!(invalid_stats.coefficient_of_variation < TIMING_VARIANCE_THRESHOLD,
            "Invalid signature verification not constant-time: CV = {:.6}", invalid_stats.coefficient_of_variation);
        
        // Timing should be similar regardless of signature validity
        let (t_stat, similar, analysis) = timing_statistics::compare_timing_distributions(
            &valid_timings, &invalid_timings
        );
        
        assert!(similar,
            "Verification timing differs between valid/invalid signatures: {}", analysis);
    }

    #[test]
    fn test_remote_timing_attack_resistance() {
        let (pk, sk) = MlKem768::keygen().unwrap();
        
        // Simulate remote timing attack on decapsulation
        let valid_ct = {
            let (ct, _ss) = MlKem768::encapsulate(&pk).unwrap();
            ct
        };
        
        let valid_remote_timings = remote_timing::simulate_remote_attack(
            {
                let sk = sk.clone();
                let ct = valid_ct.clone();
                move || MlKem768::decapsulate(&sk, &ct).unwrap_or_default().as_bytes().to_vec()
            },
            500
        );
        
        let invalid_remote_timings = remote_timing::simulate_remote_attack(
            {
                let sk = sk.clone();
                move || {
                    let mut invalid_ct_bytes = vec![0u8; 1088];
                    thread_rng().fill_bytes(&mut invalid_ct_bytes);
                    let invalid_ct = qudag_crypto::kem::Ciphertext::from_bytes(&invalid_ct_bytes);
                    MlKem768::decapsulate(&sk, &invalid_ct).unwrap_or_default().as_bytes().to_vec()
                }
            },
            500
        );
        
        // Even with network noise, timing patterns should not reveal information
        let (t_stat, similar, analysis) = timing_statistics::compare_timing_distributions(
            &valid_remote_timings, &invalid_remote_timings
        );
        
        println!("Remote timing attack analysis: {}", analysis);
        
        // With network delays, timing differences should be masked
        assert!(similar || t_stat.abs() < 5.0, // More lenient threshold for remote timing
            "Remote timing attack may be possible: {}", analysis);
    }

    #[test]
    fn test_cache_line_timing_independence() {
        precision_timing::warmup_cpu();
        
        let (pk, sk) = MlKem768::keygen().unwrap();
        
        // Test decapsulation with ciphertexts designed to hit different cache lines
        let mut cache_aligned_timings = Vec::new();
        let mut cache_misaligned_timings = Vec::new();
        
        for _ in 0..1000 {
            // Create ciphertext with cache-aligned patterns
            let mut aligned_ct_bytes = vec![0u8; 1088];
            for i in (0..aligned_ct_bytes.len()).step_by(64) { // 64-byte cache line alignment
                if i < aligned_ct_bytes.len() {
                    aligned_ct_bytes[i] = 0x42;
                }
            }
            let aligned_ct = qudag_crypto::kem::Ciphertext::from_bytes(&aligned_ct_bytes);
            
            let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                let _ = MlKem768::decapsulate(&sk, &aligned_ct);
            });
            cache_aligned_timings.push(duration);
            
            // Create ciphertext with cache-misaligned patterns
            let mut misaligned_ct_bytes = vec![0u8; 1088];
            for i in (0..misaligned_ct_bytes.len()).step_by(63) { // Misaligned to cache lines
                if i < misaligned_ct_bytes.len() {
                    misaligned_ct_bytes[i] = 0x42;
                }
            }
            let misaligned_ct = qudag_crypto::kem::Ciphertext::from_bytes(&misaligned_ct_bytes);
            
            let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                let _ = MlKem768::decapsulate(&sk, &misaligned_ct);
            });
            cache_misaligned_timings.push(duration);
        }
        
        let (t_stat, similar, analysis) = timing_statistics::compare_timing_distributions(
            &cache_aligned_timings, &cache_misaligned_timings
        );
        
        assert!(similar,
            "Cache line timing dependency detected: {}", analysis);
    }

    #[test]
    fn test_branch_prediction_timing_independence() {
        precision_timing::warmup_cpu();
        
        let keypair = MlDsa::keygen().unwrap();
        
        // Test with predictable message patterns (should trigger branch predictor)
        let mut predictable_timings = Vec::new();
        for i in 0..1000 {
            let message = vec![((i % 256) as u8); 64]; // Predictable pattern
            
            let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                MlDsa::sign(&message, keypair.secret_key()).unwrap()
            });
            predictable_timings.push(duration);
        }
        
        // Test with random message patterns (should defeat branch predictor)
        let mut random_timings = Vec::new();
        for _ in 0..1000 {
            let mut message = vec![0u8; 64];
            thread_rng().fill_bytes(&mut message);
            
            let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                MlDsa::sign(&message, keypair.secret_key()).unwrap()
            });
            random_timings.push(duration);
        }
        
        let (t_stat, similar, analysis) = timing_statistics::compare_timing_distributions(
            &predictable_timings, &random_timings
        );
        
        assert!(similar,
            "Branch prediction timing dependency detected: {}", analysis);
    }

    #[test]
    fn test_memory_access_pattern_timing() {
        precision_timing::warmup_cpu();
        
        let (pk, sk) = MlKem768::keygen().unwrap();
        
        // Test with ciphertexts that would cause different memory access patterns
        let mut sequential_timings = Vec::new();
        let mut random_timings = Vec::new();
        
        for _ in 0..1000 {
            // Sequential pattern
            let mut seq_ct_bytes = vec![0u8; 1088];
            for (i, byte) in seq_ct_bytes.iter_mut().enumerate() {
                *byte = (i % 256) as u8;
            }
            let seq_ct = qudag_crypto::kem::Ciphertext::from_bytes(&seq_ct_bytes);
            
            let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                let _ = MlKem768::decapsulate(&sk, &seq_ct);
            });
            sequential_timings.push(duration);
            
            // Random pattern
            let mut rand_ct_bytes = vec![0u8; 1088];
            thread_rng().fill_bytes(&mut rand_ct_bytes);
            let rand_ct = qudag_crypto::kem::Ciphertext::from_bytes(&rand_ct_bytes);
            
            let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                let _ = MlKem768::decapsulate(&sk, &rand_ct);
            });
            random_timings.push(duration);
        }
        
        let (t_stat, similar, analysis) = timing_statistics::compare_timing_distributions(
            &sequential_timings, &random_timings
        );
        
        assert!(similar,
            "Memory access pattern timing dependency detected: {}", analysis);
    }

    #[test]
    fn test_multi_core_timing_consistency() {
        use std::sync::Arc;
        use std::thread;
        
        precision_timing::warmup_cpu();
        
        let keypair = Arc::new(MlDsa::keygen().unwrap());
        let message = b"multi-core timing test message";
        
        let num_threads = 4;
        let samples_per_thread = 250;
        let mut handles = Vec::new();
        
        for thread_id in 0..num_threads {
            let keypair_clone = Arc::clone(&keypair);
            
            let handle = thread::spawn(move || {
                let mut timings = Vec::new();
                
                // Pin thread to specific CPU core if possible
                #[cfg(target_os = "linux")]
                {
                    use libc::{cpu_set_t, sched_setaffinity, CPU_SET, CPU_ZERO};
                    unsafe {
                        let mut cpu_set: cpu_set_t = std::mem::zeroed();
                        CPU_ZERO(&mut cpu_set);
                        CPU_SET(thread_id, &mut cpu_set);
                        sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpu_set);
                    }
                }
                
                for _ in 0..samples_per_thread {
                    let (_, duration, _cycles) = precision_timing::measure_multi_source(|| {
                        MlDsa::sign(message, keypair_clone.secret_key()).unwrap()
                    });
                    timings.push(duration);
                }
                
                (thread_id, timings)
            });
            
            handles.push(handle);
        }
        
        let mut all_thread_timings = Vec::new();
        for handle in handles {
            let (thread_id, timings) = handle.join().unwrap();
            println!("Thread {} completed {} measurements", thread_id, timings.len());
            all_thread_timings.push(timings);
        }
        
        // Compare timing distributions across threads
        let first_thread_timings = &all_thread_timings[0];
        for (i, thread_timings) in all_thread_timings.iter().enumerate().skip(1) {
            let (t_stat, similar, analysis) = timing_statistics::compare_timing_distributions(
                first_thread_timings, thread_timings
            );
            
            assert!(similar,
                "Timing differs between thread 0 and thread {}: {}", i, analysis);
        }
        
        // Overall timing consistency
        let all_timings: Vec<Duration> = all_thread_timings.into_iter().flatten().collect();
        let stats = timing_statistics::calculate_timing_stats(&all_timings);
        
        assert!(stats.coefficient_of_variation < TIMING_VARIANCE_THRESHOLD * 1.5, // Allow slightly more variance across cores
            "Multi-core timing not consistent: CV = {:.6}", stats.coefficient_of_variation);
    }
}