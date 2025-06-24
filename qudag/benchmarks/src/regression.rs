//! Performance regression detection and analysis.
//!
//! This module provides tools for detecting performance regressions by comparing
//! current measurements against established baselines.

use crate::baseline::{BaselineCollection, PerformanceBaseline};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Result of a regression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionResult {
    /// Name of the benchmark
    pub benchmark_name: String,
    /// Whether a regression was detected
    pub is_regression: bool,
    /// Current measurement value
    pub current_value: f64,
    /// Baseline value for comparison
    pub baseline_value: f64,
    /// Percentage change from baseline
    pub percentage_change: f64,
    /// Threshold used for detection
    pub threshold: f64,
    /// Severity level of the regression
    pub severity: RegressionSeverity,
    /// Additional details about the analysis
    pub details: String,
}

/// Severity levels for performance regressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegressionSeverity {
    /// No regression detected
    None,
    /// Minor regression (within acceptable bounds)
    Minor,
    /// Moderate regression (noticeable but not critical)
    Moderate,
    /// Major regression (significant performance impact)
    Major,
    /// Critical regression (severe performance degradation)
    Critical,
}

impl RegressionSeverity {
    /// Get severity from percentage change
    pub fn from_percentage(percentage: f64) -> Self {
        match percentage.abs() {
            p if p < 5.0 => Self::None,
            p if p < 15.0 => Self::Minor,
            p if p < 30.0 => Self::Moderate,
            p if p < 50.0 => Self::Major,
            _ => Self::Critical,
        }
    }

    /// Get color code for display
    pub fn color_code(&self) -> &'static str {
        match self {
            Self::None => "green",
            Self::Minor => "yellow",
            Self::Moderate => "orange",
            Self::Major => "red",
            Self::Critical => "darkred",
        }
    }
}

/// Configuration for regression detection
#[derive(Debug, Clone)]
pub struct RegressionConfig {
    /// Default threshold for regression detection (percentage)
    pub default_threshold: f64,
    /// Benchmark-specific thresholds
    pub benchmark_thresholds: HashMap<String, f64>,
    /// Whether to use adaptive thresholds based on baseline std_dev
    pub use_adaptive_thresholds: bool,
    /// Multiplier for adaptive thresholds
    pub adaptive_multiplier: f64,
}

impl Default for RegressionConfig {
    fn default() -> Self {
        Self {
            default_threshold: 10.0, // 10% default threshold
            benchmark_thresholds: HashMap::new(),
            use_adaptive_thresholds: true,
            adaptive_multiplier: 2.0, // 2 standard deviations
        }
    }
}

/// Performance regression detector
pub struct RegressionDetector {
    config: RegressionConfig,
    baselines: BaselineCollection,
}

impl RegressionDetector {
    /// Create a new regression detector
    pub fn new(baselines: BaselineCollection) -> Self {
        Self {
            config: RegressionConfig::default(),
            baselines,
        }
    }

    /// Create a new regression detector with custom configuration
    pub fn with_config(baselines: BaselineCollection, config: RegressionConfig) -> Self {
        Self { config, baselines }
    }

    /// Analyze a single measurement for regression
    pub fn analyze_measurement(
        &self,
        benchmark_name: &str,
        current_value: f64,
        metric: &str,
    ) -> Option<RegressionResult> {
        let baseline = self.baselines.get_baseline(benchmark_name)?;

        // Ensure we're comparing the same metric
        if baseline.metric != metric {
            return None;
        }

        let threshold = self.get_threshold(benchmark_name, baseline);
        let percentage_change = ((current_value - baseline.value) / baseline.value) * 100.0;

        // For latency metrics, higher values are regressions
        // For throughput metrics, lower values are regressions
        let is_regression = match metric {
            "latency" | "memory" => percentage_change > threshold,
            "throughput" => percentage_change < -threshold,
            _ => percentage_change.abs() > threshold,
        };

        let severity = if is_regression {
            RegressionSeverity::from_percentage(percentage_change)
        } else {
            RegressionSeverity::None
        };

        let details = format!(
            "Current: {:.2} {}, Baseline: {:.2} {}, Change: {:.2}%, Threshold: {:.2}%",
            current_value,
            baseline.unit,
            baseline.value,
            baseline.unit,
            percentage_change,
            threshold
        );

        Some(RegressionResult {
            benchmark_name: benchmark_name.to_string(),
            is_regression,
            current_value,
            baseline_value: baseline.value,
            percentage_change,
            threshold,
            severity,
            details,
        })
    }

    /// Analyze multiple measurements for regression
    pub fn analyze_measurements(
        &self,
        measurements: &HashMap<String, (f64, String)>, // (value, metric)
    ) -> Vec<RegressionResult> {
        measurements
            .iter()
            .filter_map(|(name, (value, metric))| self.analyze_measurement(name, *value, metric))
            .collect()
    }

    /// Get the appropriate threshold for a benchmark
    fn get_threshold(&self, benchmark_name: &str, baseline: &PerformanceBaseline) -> f64 {
        // Use benchmark-specific threshold if available
        if let Some(&threshold) = self.config.benchmark_thresholds.get(benchmark_name) {
            return threshold;
        }

        // Use adaptive threshold based on standard deviation
        if self.config.use_adaptive_thresholds && baseline.std_dev > 0.0 {
            let adaptive_threshold =
                (baseline.std_dev / baseline.value) * 100.0 * self.config.adaptive_multiplier;
            return adaptive_threshold.max(self.config.default_threshold);
        }

        self.config.default_threshold
    }

    /// Generate a regression report
    pub fn generate_report(&self, results: &[RegressionResult]) -> RegressionReport {
        let total_benchmarks = results.len();
        let regressions = results.iter().filter(|r| r.is_regression).count();
        let critical_regressions = results
            .iter()
            .filter(|r| r.severity == RegressionSeverity::Critical)
            .count();
        let major_regressions = results
            .iter()
            .filter(|r| r.severity == RegressionSeverity::Major)
            .count();

        let mut worst_regression: Option<RegressionResult> = None;
        let mut best_improvement: Option<RegressionResult> = None;

        for result in results {
            if result.is_regression {
                if worst_regression.is_none()
                    || result.percentage_change.abs()
                        > worst_regression.as_ref().unwrap().percentage_change.abs()
                {
                    worst_regression = Some(result.clone());
                }
            } else if result.percentage_change < 0.0 {
                // Improvement (negative change for latency)
                if best_improvement.is_none()
                    || result.percentage_change
                        < best_improvement.as_ref().unwrap().percentage_change
                {
                    best_improvement = Some(result.clone());
                }
            }
        }

        RegressionReport {
            timestamp: SystemTime::now(),
            total_benchmarks,
            regressions,
            critical_regressions,
            major_regressions,
            worst_regression,
            best_improvement,
            results: results.to_vec(),
        }
    }
}

/// Comprehensive regression analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionReport {
    /// Timestamp of the analysis
    pub timestamp: SystemTime,
    /// Total number of benchmarks analyzed
    pub total_benchmarks: usize,
    /// Number of regressions detected
    pub regressions: usize,
    /// Number of critical regressions
    pub critical_regressions: usize,
    /// Number of major regressions
    pub major_regressions: usize,
    /// Worst regression found
    pub worst_regression: Option<RegressionResult>,
    /// Best improvement found
    pub best_improvement: Option<RegressionResult>,
    /// All analysis results
    pub results: Vec<RegressionResult>,
}

impl RegressionReport {
    /// Check if the report contains any regressions
    pub fn has_regressions(&self) -> bool {
        self.regressions > 0
    }

    /// Check if the report contains critical regressions
    pub fn has_critical_regressions(&self) -> bool {
        self.critical_regressions > 0
    }

    /// Get summary statistics
    pub fn summary(&self) -> String {
        format!(
            "Regression Analysis Summary:\n\
             - Total benchmarks: {}\n\
             - Regressions detected: {}\n\
             - Critical regressions: {}\n\
             - Major regressions: {}\n\
             {}{}",
            self.total_benchmarks,
            self.regressions,
            self.critical_regressions,
            self.major_regressions,
            if let Some(ref worst) = self.worst_regression {
                format!(
                    "- Worst regression: {} ({:.2}%)\n",
                    worst.benchmark_name, worst.percentage_change
                )
            } else {
                String::new()
            },
            if let Some(ref best) = self.best_improvement {
                format!(
                    "- Best improvement: {} ({:.2}%)\n",
                    best.benchmark_name, best.percentage_change
                )
            } else {
                String::new()
            }
        )
    }

    /// Export report to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Save report to file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = self.to_json()?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Continuous regression monitoring
pub struct RegressionMonitor {
    detector: RegressionDetector,
    history: Vec<RegressionReport>,
    alert_threshold: usize,
}

impl RegressionMonitor {
    /// Create a new regression monitor
    pub fn new(detector: RegressionDetector) -> Self {
        Self {
            detector,
            history: Vec::new(),
            alert_threshold: 3, // Alert after 3 consecutive regressions
        }
    }

    /// Add a new measurement and check for regressions
    pub fn check_measurement(
        &mut self,
        benchmark_name: &str,
        value: f64,
        metric: &str,
    ) -> Option<RegressionResult> {
        self.detector
            .analyze_measurement(benchmark_name, value, metric)
    }

    /// Add a full regression report to history
    pub fn add_report(&mut self, report: RegressionReport) {
        self.history.push(report);

        // Keep only last 100 reports
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }

    /// Check if we should alert based on recent history
    pub fn should_alert(&self) -> bool {
        if self.history.len() < self.alert_threshold {
            return false;
        }

        let recent_reports = &self.history[self.history.len() - self.alert_threshold..];
        recent_reports.iter().all(|r| r.has_regressions())
    }

    /// Get trend analysis for a specific benchmark
    pub fn get_trend(&self, benchmark_name: &str) -> Option<Vec<f64>> {
        let values: Vec<f64> = self
            .history
            .iter()
            .filter_map(|report| {
                report
                    .results
                    .iter()
                    .find(|r| r.benchmark_name == benchmark_name)
                    .map(|r| r.current_value)
            })
            .collect();

        if values.is_empty() {
            None
        } else {
            Some(values)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::baseline::{BaselineBuilder, BaselineCollection};

    fn create_test_baseline() -> BaselineCollection {
        let mut collection = BaselineCollection::new();
        let baseline = BaselineBuilder::new("test_benchmark", "latency")
            .add_value(100.0)
            .unit("ms")
            .build()
            .unwrap();
        collection.set_baseline(baseline);
        collection
    }

    #[test]
    fn test_regression_detection_latency() {
        let baselines = create_test_baseline();
        let detector = RegressionDetector::new(baselines);

        // Test regression (latency increased)
        let result = detector
            .analyze_measurement("test_benchmark", 120.0, "latency")
            .unwrap();
        assert!(result.is_regression);
        assert_eq!(result.percentage_change, 20.0);

        // Test improvement (latency decreased)
        let result = detector
            .analyze_measurement("test_benchmark", 80.0, "latency")
            .unwrap();
        assert!(!result.is_regression);
        assert_eq!(result.percentage_change, -20.0);
    }

    #[test]
    fn test_regression_severity() {
        assert_eq!(
            RegressionSeverity::from_percentage(3.0),
            RegressionSeverity::None
        );
        assert_eq!(
            RegressionSeverity::from_percentage(10.0),
            RegressionSeverity::Minor
        );
        assert_eq!(
            RegressionSeverity::from_percentage(25.0),
            RegressionSeverity::Moderate
        );
        assert_eq!(
            RegressionSeverity::from_percentage(40.0),
            RegressionSeverity::Major
        );
        assert_eq!(
            RegressionSeverity::from_percentage(60.0),
            RegressionSeverity::Critical
        );
    }

    #[test]
    fn test_regression_report() {
        let baselines = create_test_baseline();
        let detector = RegressionDetector::new(baselines);

        let results = vec![detector
            .analyze_measurement("test_benchmark", 120.0, "latency")
            .unwrap()];

        let report = detector.generate_report(&results);
        assert_eq!(report.total_benchmarks, 1);
        assert_eq!(report.regressions, 1);
        assert!(report.has_regressions());
    }
}
