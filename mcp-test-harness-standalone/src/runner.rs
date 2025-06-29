//! Test execution engine for the standalone MCP test harness
//! 
//! Orchestrates test execution with performance monitoring and result reporting.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use crate::config::{TestConfig, TestCase, TestSuite};
use crate::server::McpServer;
use crate::validation::TestValidator;

/// Test execution orchestrator
#[derive(Debug)]
pub struct TestRunner {
    config: TestConfig,
    server: Option<McpServer>,
    validator: TestValidator,
    output_format: String,
    
    // Execution options
    validation_only: bool,
    generate_baseline: bool,
    comprehensive: bool,
    parallel_execution: usize,
}

/// Results from test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub summary: TestSummary,
    pub suite_results: Vec<SuiteResult>,
    pub performance_data: Option<PerformanceData>,
    pub execution_metadata: ExecutionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
    pub execution_time_seconds: f64,
    pub overall_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteResult {
    pub suite_name: String,
    pub test_case_results: Vec<TestCaseResult>,
    pub suite_summary: SuiteSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteSummary {
    pub total_cases: usize,
    pub passed_cases: usize,
    pub failed_cases: usize,
    pub skipped_cases: usize,
    pub execution_time_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    pub test_id: String,
    pub test_name: String,
    pub status: TestStatus,
    pub execution_time_ms: u64,
    pub memory_usage_mb: Option<f64>,
    pub error_message: Option<String>,
    pub response: Option<serde_json::Value>,
    pub validation_results: Vec<ValidationResult>,
    pub performance_metrics: Option<PerformanceMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub rule_name: String,
    pub passed: bool,
    pub message: String,
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub execution_time_ms: u64,
    pub memory_peak_mb: Option<f64>,
    pub memory_average_mb: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
    pub network_requests: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceData {
    pub overall_metrics: PerformanceMetrics,
    pub baseline_comparison: Option<BaselineComparison>,
    pub regression_analysis: Option<RegressionAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub performance_change_percent: f64,
    pub memory_change_percent: f64,
    pub is_improvement: bool,
    pub is_regression: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    pub detected_regressions: Vec<RegressionAlert>,
    pub detected_improvements: Vec<ImprovementAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAlert {
    pub test_name: String,
    pub metric: String,
    pub change_percent: f64,
    pub severity: AlertSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementAlert {
    pub test_name: String,
    pub metric: String,
    pub improvement_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub start_time: String,
    pub end_time: String,
    pub execution_environment: String,
    pub test_harness_version: String,
    pub server_info: Option<ServerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub server_type: String,
    pub version: Option<String>,
    pub capabilities: Option<serde_json::Value>,
    pub transport: String,
}

/// Benchmark execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub benchmark_summary: BenchmarkSummary,
    pub detailed_results: Vec<BenchmarkTestResult>,
    pub performance_analysis: PerformanceAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub total_iterations: usize,
    pub duration_seconds: u64,
    pub average_ops_per_second: f64,
    pub average_response_time_ms: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTestResult {
    pub test_name: String,
    pub iterations_completed: usize,
    pub average_time_ms: f64,
    pub min_time_ms: f64,
    pub max_time_ms: f64,
    pub std_deviation_ms: f64,
    pub ops_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    pub bottlenecks: Vec<String>,
    pub recommendations: Vec<String>,
    pub resource_utilization: HashMap<String, f64>,
}

impl TestRunner {
    /// Create new test runner
    pub fn new(config: TestConfig, output_format: String) -> Result<Self> {
        let validator = TestValidator::new();
        
        Ok(Self {
            config,
            server: None,
            validator,
            output_format,
            validation_only: false,
            generate_baseline: false,
            comprehensive: false,
            parallel_execution: 1,
        })
    }
    
    /// Set server command for stdio servers
    pub fn set_server_command(&mut self, command: String, working_dir: Option<PathBuf>) {
        self.config.server.command = Some(command);
        if let Some(dir) = working_dir {
            self.config.server.working_dir = Some(dir.to_string_lossy().to_string());
        }
    }
    
    /// Set execution options
    pub fn set_validation_only(&mut self, validation_only: bool) {
        self.validation_only = validation_only;
    }
    
    pub fn set_generate_baseline(&mut self, generate_baseline: bool) {
        self.generate_baseline = generate_baseline;
    }
    
    pub fn set_comprehensive(&mut self, comprehensive: bool) {
        self.comprehensive = comprehensive;
    }
    
    pub fn set_parallel_execution(&mut self, parallel: usize) {
        self.parallel_execution = parallel;
    }
    
    /// Run all tests
    pub async fn run(&mut self) -> Result<TestResults> {
        let start_time = Instant::now();
        info!("Starting test execution with {} test suites", self.config.test_suites.len());
        
        // Initialize server if not validation-only
        if !self.validation_only {
            self.initialize_server().await?;
        }
        
        let mut suite_results = Vec::new();
        let mut total_tests = 0;
        let mut passed_tests = 0;
        let mut failed_tests = 0;
        let mut skipped_tests = 0;
        
        // Clone test suites to avoid borrow checker issues
        let test_suites = self.config.test_suites.clone();
        
        // Execute test suites
        for suite in &test_suites {
            info!("Executing test suite: {}", suite.name);
            
            let suite_result = if self.parallel_execution > 1 {
                self.run_suite_parallel(suite).await?
            } else {
                self.run_suite_sequential(suite).await?
            };
            
            total_tests += suite_result.suite_summary.total_cases;
            passed_tests += suite_result.suite_summary.passed_cases;
            failed_tests += suite_result.suite_summary.failed_cases;
            skipped_tests += suite_result.suite_summary.skipped_cases;
            
            suite_results.push(suite_result);
        }
        
        // Clean up server
        if let Some(ref mut server) = self.server {
            server.stop().await?;
        }
        
        let execution_time = start_time.elapsed();
        let overall_success_rate = if total_tests > 0 {
            passed_tests as f64 / total_tests as f64
        } else {
            0.0
        };
        
        let results = TestResults {
            summary: TestSummary {
                total_tests,
                passed_tests,
                failed_tests,
                skipped_tests,
                execution_time_seconds: execution_time.as_secs_f64(),
                overall_success_rate,
            },
            suite_results,
            performance_data: None, // Would be populated with actual performance monitoring
            execution_metadata: ExecutionMetadata {
                start_time: chrono::Utc::now().to_rfc3339(),
                end_time: chrono::Utc::now().to_rfc3339(),
                execution_environment: format!("{}_{}", std::env::consts::OS, std::env::consts::ARCH),
                test_harness_version: env!("CARGO_PKG_VERSION").to_string(),
                server_info: None, // Would be populated from server introspection
            },
        };
        
        info!("Test execution completed: {}/{} passed", passed_tests, total_tests);
        Ok(results)
    }
    
    /// Run benchmark tests
    pub async fn run_benchmark(&mut self, iterations: usize, duration: u64) -> Result<BenchmarkResults> {
        info!("Starting benchmark with {} iterations over {}s", iterations, duration);
        
        if !self.validation_only {
            self.initialize_server().await?;
        }
        
        let start_time = Instant::now();
        let duration_limit = Duration::from_secs(duration);
        let mut benchmark_results = Vec::new();
        let mut total_iterations = 0;
        let mut total_successes = 0;
        
        // Clone test suites to avoid borrow checker issues
        let test_suites = self.config.test_suites.clone();
        
        // Run benchmarks for each test case
        for suite in &test_suites {
            for test_case in &suite.test_cases {
                if !test_case.enabled {
                    continue;
                }
                
                let test_benchmark = self.benchmark_test_case(test_case, iterations, duration_limit).await?;
                total_iterations += test_benchmark.iterations_completed;
                if test_benchmark.ops_per_second > 0.0 {
                    total_successes += test_benchmark.iterations_completed;
                }
                benchmark_results.push(test_benchmark);
            }
        }
        
        let total_duration = start_time.elapsed();
        let average_ops_per_second = if total_duration.as_secs_f64() > 0.0 {
            total_iterations as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };
        
        let average_response_time = if !benchmark_results.is_empty() {
            benchmark_results.iter().map(|r| r.average_time_ms).sum::<f64>() / benchmark_results.len() as f64
        } else {
            0.0
        };
        
        let success_rate = if total_iterations > 0 {
            total_successes as f64 / total_iterations as f64
        } else {
            0.0
        };
        
        Ok(BenchmarkResults {
            benchmark_summary: BenchmarkSummary {
                total_iterations,
                duration_seconds: duration,
                average_ops_per_second,
                average_response_time_ms: average_response_time,
                success_rate,
            },
            detailed_results: benchmark_results,
            performance_analysis: PerformanceAnalysis {
                bottlenecks: vec![], // Would be populated with actual analysis
                recommendations: vec![], // Would be populated with recommendations
                resource_utilization: HashMap::new(),
            },
        })
    }
    
    async fn initialize_server(&mut self) -> Result<()> {
        info!("Initializing MCP server");
        
        let mut server = McpServer::new(self.config.server.clone());
        server.start().await?;
        
        // Perform health check
        if !server.health_check().await? {
            return Err(anyhow!("Server failed health check"));
        }
        
        self.server = Some(server);
        Ok(())
    }
    
    async fn run_suite_sequential(&mut self, suite: &TestSuite) -> Result<SuiteResult> {
        let start_time = Instant::now();
        let mut test_case_results = Vec::new();
        let mut passed_cases = 0;
        let mut failed_cases = 0;
        let mut skipped_cases = 0;
        
        for test_case in &suite.test_cases {
            let result = self.run_test_case(test_case).await?;
            
            match result.status {
                TestStatus::Passed => passed_cases += 1,
                TestStatus::Failed | TestStatus::Error => failed_cases += 1,
                TestStatus::Skipped => skipped_cases += 1,
            }
            
            test_case_results.push(result);
            
            // Check fail_fast after cloning to avoid borrow issues
            if self.config.global.fail_fast && failed_cases > 0 {
                warn!("Fail-fast enabled, stopping suite execution");
                break;
            }
        }
        
        Ok(SuiteResult {
            suite_name: suite.name.clone(),
            test_case_results,
            suite_summary: SuiteSummary {
                total_cases: suite.test_cases.len(),
                passed_cases,
                failed_cases,
                skipped_cases,
                execution_time_seconds: start_time.elapsed().as_secs_f64(),
            },
        })
    }
    
    async fn run_suite_parallel(&mut self, suite: &TestSuite) -> Result<SuiteResult> {
        // Simplified parallel implementation - would use proper async concurrency
        info!("Running suite '{}' with parallel execution ({})", suite.name, self.parallel_execution);
        
        // For now, fall back to sequential execution
        // Real implementation would use tokio::spawn and semaphores for concurrency control
        self.run_suite_sequential(suite).await
    }
    
    async fn run_test_case(&mut self, test_case: &TestCase) -> Result<TestCaseResult> {
        if !test_case.enabled {
            return Ok(TestCaseResult {
                test_id: test_case.id.clone(),
                test_name: test_case.tool_name.clone(),
                status: TestStatus::Skipped,
                execution_time_ms: 0,
                memory_usage_mb: None,
                error_message: Some("Test case disabled".to_string()),
                response: None,
                validation_results: vec![],
                performance_metrics: None,
            });
        }
        
        debug!("Running test case: {} ({})", test_case.id, test_case.tool_name);
        let start_time = Instant::now();
        
        // Validation-only mode
        if self.validation_only {
            let validation_results = self.validator.validate_test_case(test_case);
            return Ok(TestCaseResult {
                test_id: test_case.id.clone(),
                test_name: test_case.tool_name.clone(),
                status: if validation_results.iter().all(|r| r.passed) {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                },
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                memory_usage_mb: None,
                error_message: None,
                response: None,
                validation_results,
                performance_metrics: None,
            });
        }
        
        // Execute actual test
        let result = match self.execute_test_case(test_case).await {
            Ok(response) => {
                let validation_results = self.validator.validate_response(test_case, &response);
                let status = if validation_results.iter().all(|r| r.passed) {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                };
                
                TestCaseResult {
                    test_id: test_case.id.clone(),
                    test_name: test_case.tool_name.clone(),
                    status,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    memory_usage_mb: None, // Would be populated with actual monitoring
                    error_message: None,
                    response: Some(response),
                    validation_results,
                    performance_metrics: None, // Would be populated with actual metrics
                }
            }
            Err(e) => {
                TestCaseResult {
                    test_id: test_case.id.clone(),
                    test_name: test_case.tool_name.clone(),
                    status: TestStatus::Error,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    memory_usage_mb: None,
                    error_message: Some(e.to_string()),
                    response: None,
                    validation_results: vec![],
                    performance_metrics: None,
                }
            }
        };
        
        Ok(result)
    }
    
    async fn execute_test_case(&mut self, test_case: &TestCase) -> Result<serde_json::Value> {
        let server = self.server.as_mut()
            .ok_or_else(|| anyhow!("Server not initialized"))?;
        
        server.send_request(&test_case.tool_name, test_case.input_params.clone()).await
    }
    
    async fn benchmark_test_case(
        &mut self,
        test_case: &TestCase,
        max_iterations: usize,
        duration_limit: Duration,
    ) -> Result<BenchmarkTestResult> {
        debug!("Benchmarking test case: {}", test_case.id);
        
        let mut times = Vec::new();
        let mut iterations = 0;
        let start_time = Instant::now();
        
        while iterations < max_iterations && start_time.elapsed() < duration_limit {
            let iteration_start = Instant::now();
            
            match self.execute_test_case(test_case).await {
                Ok(_) => {
                    times.push(iteration_start.elapsed().as_millis() as f64);
                }
                Err(_) => {
                    // Count failed iterations but don't include in timing
                }
            }
            
            iterations += 1;
        }
        
        let average_time = if !times.is_empty() {
            times.iter().sum::<f64>() / times.len() as f64
        } else {
            0.0
        };
        
        let min_time = times.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_time = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        
        let variance = if times.len() > 1 {
            let mean = average_time;
            times.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / (times.len() - 1) as f64
        } else {
            0.0
        };
        let std_deviation = variance.sqrt();
        
        let ops_per_second = if average_time > 0.0 {
            1000.0 / average_time
        } else {
            0.0
        };
        
        Ok(BenchmarkTestResult {
            test_name: test_case.id.clone(),
            iterations_completed: iterations,
            average_time_ms: average_time,
            min_time_ms: if min_time.is_finite() { min_time } else { 0.0 },
            max_time_ms: if max_time.is_finite() { max_time } else { 0.0 },
            std_deviation_ms: std_deviation,
            ops_per_second,
        })
    }
}

impl TestResults {
    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.summary.failed_tests == 0
    }
    
    /// Display results in table format
    pub fn display_table(&self) {
        println!("\n📊 Test Execution Summary");
        println!("═══════════════════════════════════════");
        println!("Total Tests:    {}", self.summary.total_tests);
        println!("Passed:         {} ✅", self.summary.passed_tests);
        println!("Failed:         {} ❌", self.summary.failed_tests);
        println!("Skipped:        {} ⏭️", self.summary.skipped_tests);
        println!("Success Rate:   {:.1}%", self.summary.overall_success_rate * 100.0);
        println!("Execution Time: {:.2}s", self.summary.execution_time_seconds);
        
        for suite_result in &self.suite_results {
            println!("\n📁 {}", suite_result.suite_name);
            println!("───────────────────────────────────────");
            
            for test_result in &suite_result.test_case_results {
                let status_icon = match test_result.status {
                    TestStatus::Passed => "✅",
                    TestStatus::Failed => "❌",
                    TestStatus::Skipped => "⏭️",
                    TestStatus::Error => "💥",
                };
                
                println!(
                    "  {} {} ({}ms)",
                    status_icon,
                    test_result.test_id,
                    test_result.execution_time_ms
                );
                
                if let Some(ref error) = test_result.error_message {
                    println!("    Error: {}", error);
                }
            }
        }
    }
}

impl BenchmarkResults {
    /// Display benchmark results in table format
    pub fn display_table(&self) {
        println!("\n🏃 Benchmark Results");
        println!("═══════════════════════════════════════");
        println!("Total Iterations: {}", self.benchmark_summary.total_iterations);
        println!("Duration:         {}s", self.benchmark_summary.duration_seconds);
        println!("Avg Ops/sec:      {:.2}", self.benchmark_summary.average_ops_per_second);
        println!("Avg Response:     {:.2}ms", self.benchmark_summary.average_response_time_ms);
        println!("Success Rate:     {:.1}%", self.benchmark_summary.success_rate * 100.0);
        
        println!("\n📋 Detailed Results");
        println!("───────────────────────────────────────");
        
        for result in &self.detailed_results {
            println!(
                "  📊 {} - {:.2}ms avg ({:.2} ops/sec)",
                result.test_name,
                result.average_time_ms,
                result.ops_per_second
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TestConfig;
    
    #[tokio::test]
    async fn test_runner_creation() {
        let config = TestConfig::default_config();
        let runner = TestRunner::new(config, "table".to_string());
        assert!(runner.is_ok());
    }
    
    #[test]
    fn test_results_serialization() {
        let results = TestResults {
            summary: TestSummary {
                total_tests: 10,
                passed_tests: 8,
                failed_tests: 2,
                skipped_tests: 0,
                execution_time_seconds: 30.5,
                overall_success_rate: 0.8,
            },
            suite_results: vec![],
            performance_data: None,
            execution_metadata: ExecutionMetadata {
                start_time: "2024-01-01T00:00:00Z".to_string(),
                end_time: "2024-01-01T00:00:30Z".to_string(),
                execution_environment: "linux_x86_64".to_string(),
                test_harness_version: "0.1.0".to_string(),
                server_info: None,
            },
        };
        
        let json = serde_json::to_string(&results).unwrap();
        let _deserialized: TestResults = serde_json::from_str(&json).unwrap();
    }
}
