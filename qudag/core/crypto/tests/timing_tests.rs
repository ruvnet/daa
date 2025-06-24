use qudag_crypto::ml_kem::*;
use rand::RngCore;
use std::time::{Duration, Instant};

/// Measures execution time of a function
fn measure_time<F, T>(f: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

#[test]
fn test_constant_time_keypair_generation() {
    let mut rng = rand::thread_rng();
    let mut times = Vec::with_capacity(100);

    // Generate multiple keypairs and measure times
    for _ in 0..100 {
        let (_, duration) = measure_time(|| generate_keypair(&mut rng));
        times.push(duration);
    }

    // Calculate statistics
    let mean = times.iter().sum::<Duration>() / times.len() as u32;
    let max_deviation = times
        .iter()
        .map(|&t| if t > mean { t - mean } else { mean - t })
        .max()
        .unwrap();

    // Verify timing consistency (within 1ms)
    assert!(max_deviation < Duration::from_millis(1));
}

#[test]
fn test_constant_time_encapsulation() {
    let mut rng = rand::thread_rng();
    let keypair = generate_keypair(&mut rng).unwrap();
    let mut times = Vec::with_capacity(100);

    // Measure multiple encapsulations
    for _ in 0..100 {
        let (_, duration) = measure_time(|| encapsulate(&keypair.public_key));
        times.push(duration);
    }

    let mean = times.iter().sum::<Duration>() / times.len() as u32;
    let max_deviation = times
        .iter()
        .map(|&t| if t > mean { t - mean } else { mean - t })
        .max()
        .unwrap();

    assert!(max_deviation < Duration::from_millis(1));
}

#[test]
fn test_constant_time_decapsulation() {
    let mut rng = rand::thread_rng();
    let keypair = generate_keypair(&mut rng).unwrap();
    let (_, ct) = encapsulate(&keypair.public_key).unwrap();

    let mut valid_times = Vec::with_capacity(50);
    let mut invalid_times = Vec::with_capacity(50);

    // Measure valid decapsulations
    for _ in 0..50 {
        let (_, duration) = measure_time(|| decapsulate(&keypair.secret_key, &ct));
        valid_times.push(duration);
    }

    // Measure invalid decapsulations
    let mut invalid_ct = vec![0u8; ct.len()];
    for _ in 0..50 {
        rng.fill_bytes(&mut invalid_ct);
        let (_, duration) = measure_time(|| decapsulate(&keypair.secret_key, &invalid_ct));
        invalid_times.push(duration);
    }

    // Calculate statistics
    let valid_mean = valid_times.iter().sum::<Duration>() / valid_times.len() as u32;
    let invalid_mean = invalid_times.iter().sum::<Duration>() / invalid_times.len() as u32;
    let timing_diff = if valid_mean > invalid_mean {
        valid_mean - invalid_mean
    } else {
        invalid_mean - valid_mean
    };

    // Verify constant-time behavior
    assert!(timing_diff < Duration::from_millis(1));
}

#[test]
fn test_constant_time_comparison() {
    let mut rng = rand::thread_rng();
    let mut data1 = vec![0u8; 32];
    let mut data2 = vec![0u8; 32];
    let mut times = Vec::with_capacity(100);

    // Compare equal data
    rng.fill_bytes(&mut data1);
    data2.copy_from_slice(&data1);
    for _ in 0..50 {
        let (_, duration) = measure_time(|| constant_time_compare(&data1, &data2));
        times.push(duration);
    }

    // Compare different data
    rng.fill_bytes(&mut data2);
    for _ in 0..50 {
        let (_, duration) = measure_time(|| constant_time_compare(&data1, &data2));
        times.push(duration);
    }

    let mean = times.iter().sum::<Duration>() / times.len() as u32;
    let max_deviation = times
        .iter()
        .map(|&t| if t > mean { t - mean } else { mean - t })
        .max()
        .unwrap();

    assert!(max_deviation < Duration::from_millis(1));
}
