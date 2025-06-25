//! Comprehensive test runner for all DAA Compute tests

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde_json::{json, Value};
use tokio;

#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration: Duration,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TestSuite {
    pub name: String,
    pub tests: Vec<TestResult>,
    pub total_duration: Duration,
    pub pass_rate: f64,
}

pub struct TestRunner {
    pub results: Vec<TestSuite>,
    pub start_time: Instant,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            start_time: Instant::now(),
        }
    }

    pub async fn run_all_tests(&mut self) -> anyhow::Result<()> {
        println!("🚀 Starting comprehensive DAA Compute test suite...");
        println!("=" * 80);

        // Run unit tests
        self.run_unit_tests().await?;
        
        // Run integration tests
        self.run_integration_tests().await?;
        
        // Run simulation tests
        self.run_simulation_tests().await?;
        
        // Run benchmark tests
        self.run_benchmark_tests().await?;
        
        // Generate final report
        self.generate_report().await?;
        
        Ok(())
    }

    async fn run_unit_tests(&mut self) -> anyhow::Result<()> {
        println!("\n📋 Running Unit Tests");
        println!("-" * 40);
        
        let unit_tests = vec![
            ("P2P Gradient Tests", "tests/unit/p2p/test_gradient.rs"),
            ("P2P Compression Tests", "tests/unit/p2p/test_compression.rs"),
            ("Protocol Aggregation Tests", "tests/unit/protocols/test_aggregation.rs"),
        ];
        
        let mut suite_results = Vec::new();
        let suite_start = Instant::now();
        
        for (name, test_file) in unit_tests {
            println!("  Running: {}", name);
            let result = self.run_test_file(test_file).await?;
            suite_results.push(result);
        }
        
        let suite = TestSuite {
            name: "Unit Tests".to_string(),
            tests: suite_results,
            total_duration: suite_start.elapsed(),
            pass_rate: self.calculate_pass_rate(&suite_results),
        };
        
        self.results.push(suite);
        Ok(())
    }

    async fn run_integration_tests(&mut self) -> anyhow::Result<()> {
        println!("\n🔗 Running Integration Tests");
        println!("-" * 40);
        
        let integration_tests = vec![
            ("P2P Networking Integration", "tests/integration/test_p2p_networking.rs"),
        ];
        
        let mut suite_results = Vec::new();
        let suite_start = Instant::now();
        
        for (name, test_file) in integration_tests {
            println!("  Running: {}", name);
            let result = self.run_test_file(test_file).await?;
            suite_results.push(result);
        }
        
        let suite = TestSuite {
            name: "Integration Tests".to_string(),
            tests: suite_results,
            total_duration: suite_start.elapsed(),
            pass_rate: self.calculate_pass_rate(&suite_results),
        };
        
        self.results.push(suite);
        Ok(())
    }

    async fn run_simulation_tests(&mut self) -> anyhow::Result<()> {
        println!("\n🌊 Running Simulation Tests");
        println!("-" * 40);
        
        let simulation_tests = vec![
            ("Node Churn Simulation", "tests/simulation/test_node_churn.rs"),
        ];
        
        let mut suite_results = Vec::new();
        let suite_start = Instant::now();
        
        for (name, test_file) in simulation_tests {
            println!("  Running: {}", name);
            let result = self.run_test_file(test_file).await?;
            suite_results.push(result);
        }
        
        let suite = TestSuite {
            name: "Simulation Tests".to_string(),
            tests: suite_results,
            total_duration: suite_start.elapsed(),
            pass_rate: self.calculate_pass_rate(&suite_results),
        };
        
        self.results.push(suite);
        Ok(())
    }

    async fn run_benchmark_tests(&mut self) -> anyhow::Result<()> {
        println!("\n⚡ Running Benchmark Tests");
        println!("-" * 40);
        
        let benchmark_tests = vec![
            ("Training Performance Benchmarks", "tests/benchmarks/test_training_performance.rs"),
        ];
        
        let mut suite_results = Vec::new();
        let suite_start = Instant::now();
        
        for (name, test_file) in benchmark_tests {
            println!("  Running: {}", name);
            let result = self.run_test_file(test_file).await?;
            suite_results.push(result);
        }
        
        let suite = TestSuite {
            name: "Benchmark Tests".to_string(),
            tests: suite_results,
            total_duration: suite_start.elapsed(),
            pass_rate: self.calculate_pass_rate(&suite_results),
        };
        
        self.results.push(suite);
        Ok(())
    }

    async fn run_test_file(&self, test_file: &str) -> anyhow::Result<TestResult> {
        let start = Instant::now();
        
        // Build the test
        let build_output = Command::new("cargo")
            .args(&["test", "--no-run", "--test", &test_file.replace("tests/", "").replace(".rs", "")])
            .current_dir("/workspaces/daa/daa-compute")
            .output()?;
        
        if !build_output.status.success() {
            return Ok(TestResult {
                name: test_file.to_string(),
                passed: false,
                duration: start.elapsed(),
                output: String::from_utf8_lossy(&build_output.stdout).to_string(),
                error: Some(String::from_utf8_lossy(&build_output.stderr).to_string()),
            });
        }
        
        // Run the test
        let test_output = Command::new("cargo")
            .args(&["test", "--test", &test_file.replace("tests/", "").replace(".rs", ""), "--", "--nocapture"])
            .current_dir("/workspaces/daa/daa-compute")
            .output()?;
        
        let passed = test_output.status.success();
        let output = String::from_utf8_lossy(&test_output.stdout).to_string();
        let error = if test_output.stderr.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&test_output.stderr).to_string())
        };
        
        let duration = start.elapsed();
        
        // Print immediate result
        let status = if passed { "✅ PASS" } else { "❌ FAIL" };
        println!("    {} ({:?})", status, duration);
        
        if !passed {
            println!("    Error: {}", error.as_ref().unwrap_or(&"Unknown error".to_string()));
        }
        
        Ok(TestResult {
            name: test_file.to_string(),
            passed,
            duration,
            output,
            error,
        })
    }

    fn calculate_pass_rate(&self, results: &[TestResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        
        let passed = results.iter().filter(|r| r.passed).count();
        (passed as f64 / results.len() as f64) * 100.0
    }

    async fn generate_report(&self) -> anyhow::Result<()> {
        let total_duration = self.start_time.elapsed();
        
        println!("\n" + &"=".repeat(80));
        println!("📊 COMPREHENSIVE TEST REPORT");
        println!("=" * 80);
        
        let mut total_tests = 0;
        let mut total_passed = 0;
        
        for suite in &self.results {
            println!("\n📋 {}", suite.name);
            println!("   Duration: {:?}", suite.total_duration);
            println!("   Pass Rate: {:.1}%", suite.pass_rate);
            println!("   Tests: {}", suite.tests.len());
            
            total_tests += suite.tests.len();
            total_passed += suite.tests.iter().filter(|t| t.passed).count();
            
            for test in &suite.tests {
                let status = if test.passed { "✅" } else { "❌" };
                println!("     {} {} ({:?})", status, test.name, test.duration);
                
                if !test.passed && test.error.is_some() {
                    println!("       Error: {}", test.error.as_ref().unwrap());
                }
            }
        }
        
        let overall_pass_rate = if total_tests > 0 {
            (total_passed as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };
        
        println!("\n" + &"=".repeat(80));
        println!("🎯 OVERALL RESULTS");
        println!("=" * 80);
        println!("Total Tests: {}", total_tests);
        println!("Passed: {}", total_passed);
        println!("Failed: {}", total_tests - total_passed);
        println!("Pass Rate: {:.1}%", overall_pass_rate);
        println!("Total Duration: {:?}", total_duration);
        
        // Generate JSON report for memory storage
        let report = self.generate_json_report(overall_pass_rate, total_duration).await?;
        
        println!("\n📝 Test report generated for memory storage");
        
        // Check if we achieved 100% pass rate
        if overall_pass_rate >= 100.0 {
            println!("\n🎉 ALL TESTS PASSED! 100% SUCCESS RATE ACHIEVED!");
        } else {
            println!("\n⚠️  Some tests failed. Pass rate: {:.1}%", overall_pass_rate);
        }
        
        Ok(())
    }

    async fn generate_json_report(&self, overall_pass_rate: f64, total_duration: Duration) -> anyhow::Result<Value> {
        let mut suites = Vec::new();
        
        for suite in &self.results {
            let mut tests = Vec::new();
            
            for test in &suite.tests {
                tests.push(json!({
                    "name": test.name,
                    "passed": test.passed,
                    "duration_ms": test.duration.as_millis(),
                    "output": test.output,
                    "error": test.error
                }));
            }
            
            suites.push(json!({
                "name": suite.name,
                "tests": tests,
                "total_duration_ms": suite.total_duration.as_millis(),
                "pass_rate": suite.pass_rate
            }));
        }
        
        let report = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "overall_pass_rate": overall_pass_rate,
            "total_duration_ms": total_duration.as_millis(),
            "test_suites": suites,
            "summary": {
                "unit_tests": self.get_suite_summary("Unit Tests"),
                "integration_tests": self.get_suite_summary("Integration Tests"),
                "simulation_tests": self.get_suite_summary("Simulation Tests"),
                "benchmark_tests": self.get_suite_summary("Benchmark Tests")
            },
            "environment": {
                "platform": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "rust_version": env!("RUSTC_VERSION")
            }
        });
        
        Ok(report)
    }

    fn get_suite_summary(&self, suite_name: &str) -> Value {
        if let Some(suite) = self.results.iter().find(|s| s.name == suite_name) {
            json!({
                "total": suite.tests.len(),
                "passed": suite.tests.iter().filter(|t| t.passed).count(),
                "pass_rate": suite.pass_rate,
                "duration_ms": suite.total_duration.as_millis()
            })
        } else {
            json!({
                "total": 0,
                "passed": 0,
                "pass_rate": 0.0,
                "duration_ms": 0
            })
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut runner = TestRunner::new();
    runner.run_all_tests().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runner_creation() {
        let runner = TestRunner::new();
        assert_eq!(runner.results.len(), 0);
    }

    #[tokio::test]
    async fn test_pass_rate_calculation() {
        let runner = TestRunner::new();
        
        let results = vec![
            TestResult {
                name: "test1".to_string(),
                passed: true,
                duration: Duration::from_millis(100),
                output: "".to_string(),
                error: None,
            },
            TestResult {
                name: "test2".to_string(),
                passed: false,
                duration: Duration::from_millis(200),
                output: "".to_string(),
                error: Some("failed".to_string()),
            },
        ];
        
        let pass_rate = runner.calculate_pass_rate(&results);
        assert_eq!(pass_rate, 50.0);
    }
}