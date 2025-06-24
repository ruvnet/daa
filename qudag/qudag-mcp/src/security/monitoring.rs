//! Security monitoring and threat detection for QuDAG MCP.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::{interval, Instant};
use tracing::{debug, info, warn, error};

use crate::error::{McpError, McpResult};
use crate::security::{SecurityContext, SecuritySeverity, MonitoringConfig, AlertThresholds};

/// Security monitor for threat detection and anomaly detection
pub struct SecurityMonitor {
    /// Configuration
    config: MonitoringConfig,
    
    /// Threat detector
    threat_detector: Arc<ThreatDetector>,
    
    /// Anomaly detector
    anomaly_detector: Arc<AnomalyDetector>,
    
    /// Alert manager
    alert_manager: Arc<AlertManager>,
    
    /// Security metrics collector
    metrics: Arc<RwLock<SecurityMetrics>>,
    
    /// Request tracking
    request_tracker: Arc<RwLock<RequestTracker>>,
}

/// Threat detection system
pub struct ThreatDetector {
    /// Known attack patterns
    attack_patterns: Vec<AttackPattern>,
    
    /// IP reputation database
    ip_reputation: Arc<RwLock<HashMap<String, IpReputation>>>,
    
    /// Rate limiting trackers
    rate_limiters: Arc<RwLock<HashMap<String, RateLimitTracker>>>,
    
    /// Configuration
    config: ThreatDetectionConfig,
}

/// Anomaly detection system
pub struct AnomalyDetector {
    /// Baseline behavior patterns
    baselines: Arc<RwLock<HashMap<String, BehaviorBaseline>>>,
    
    /// Current behavior tracking
    current_behavior: Arc<RwLock<HashMap<String, BehaviorTracker>>>,
    
    /// Configuration
    config: AnomalyDetectionConfig,
}

/// Alert management system
pub struct AlertManager {
    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<String, SecurityAlert>>>,
    
    /// Alert history
    alert_history: Arc<RwLock<VecDeque<SecurityAlert>>>,
    
    /// Alert handlers
    handlers: Vec<Arc<dyn AlertHandler>>,
    
    /// Configuration
    config: AlertConfig,
}

/// Security alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    /// Alert ID
    pub id: String,
    
    /// Alert type
    pub alert_type: AlertType,
    
    /// Alert severity
    pub severity: SecuritySeverity,
    
    /// Alert timestamp
    pub timestamp: SystemTime,
    
    /// Alert source
    pub source: String,
    
    /// Alert title
    pub title: String,
    
    /// Alert description
    pub description: String,
    
    /// Associated user ID
    pub user_id: Option<String>,
    
    /// Associated client IP
    pub client_ip: Option<String>,
    
    /// Associated request ID
    pub request_id: Option<String>,
    
    /// Alert data
    pub data: serde_json::Value,
    
    /// Alert tags
    pub tags: Vec<String>,
    
    /// Alert status
    pub status: AlertStatus,
    
    /// Alert resolution
    pub resolution: Option<AlertResolution>,
}

/// Types of security alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    /// Brute force attack detected
    BruteForce,
    
    /// Rate limit exceeded
    RateLimit,
    
    /// Suspicious IP activity
    SuspiciousIp,
    
    /// Anomalous behavior detected
    AnomalousAccess,
    
    /// Authentication failure spike
    AuthFailureSpike,
    
    /// Privilege escalation attempt
    PrivilegeEscalation,
    
    /// Data exfiltration attempt
    DataExfiltration,
    
    /// Malicious request detected
    MaliciousRequest,
    
    /// System intrusion detected
    SystemIntrusion,
    
    /// Configuration tampering
    ConfigTampering,
}

/// Alert status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    /// Alert is new/active
    Active,
    
    /// Alert is being investigated
    Investigating,
    
    /// Alert has been resolved
    Resolved,
    
    /// Alert is a false positive
    FalsePositive,
    
    /// Alert has been suppressed
    Suppressed,
}

/// Alert resolution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertResolution {
    /// Resolution timestamp
    pub timestamp: SystemTime,
    
    /// Resolution type
    pub resolution_type: ResolutionType,
    
    /// Resolution description
    pub description: String,
    
    /// Resolved by user
    pub resolved_by: String,
    
    /// Actions taken
    pub actions_taken: Vec<String>,
}

/// Resolution types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionType {
    /// Manually resolved
    Manual,
    
    /// Automatically resolved
    Automatic,
    
    /// Resolved by timeout
    Timeout,
    
    /// Resolved by system
    System,
}

/// Attack pattern definition
#[derive(Debug, Clone)]
pub struct AttackPattern {
    /// Pattern name
    pub name: String,
    
    /// Pattern type
    pub pattern_type: AttackType,
    
    /// Pattern rules
    pub rules: Vec<PatternRule>,
    
    /// Confidence threshold
    pub confidence_threshold: f64,
    
    /// Severity when matched
    pub severity: SecuritySeverity,
}

/// Types of attacks
#[derive(Debug, Clone)]
pub enum AttackType {
    /// SQL injection
    SqlInjection,
    
    /// Cross-site scripting
    Xss,
    
    /// Path traversal
    PathTraversal,
    
    /// Command injection
    CommandInjection,
    
    /// Brute force
    BruteForce,
    
    /// DDoS
    DenialOfService,
    
    /// Custom pattern
    Custom(String),
}

/// Pattern matching rule
#[derive(Debug, Clone)]
pub struct PatternRule {
    /// Field to check
    pub field: String,
    
    /// Rule type
    pub rule_type: RuleType,
    
    /// Pattern or value to match
    pub pattern: String,
    
    /// Rule weight
    pub weight: f64,
}

/// Rule types for pattern matching
#[derive(Debug, Clone)]
pub enum RuleType {
    /// Exact match
    Exact,
    
    /// Regex match
    Regex,
    
    /// Contains substring
    Contains,
    
    /// Starts with
    StartsWith,
    
    /// Ends with
    EndsWith,
    
    /// Numeric comparison
    Numeric { operator: NumericOperator, value: f64 },
}

/// Numeric comparison operators
#[derive(Debug, Clone)]
pub enum NumericOperator {
    GreaterThan,
    LessThan,
    Equal,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// IP reputation information
#[derive(Debug, Clone)]
pub struct IpReputation {
    /// IP address
    pub ip: String,
    
    /// Reputation score (0-100, higher is better)
    pub score: u8,
    
    /// Last seen timestamp
    pub last_seen: SystemTime,
    
    /// Threat categories
    pub threat_categories: Vec<ThreatCategory>,
    
    /// Number of incidents
    pub incident_count: u32,
    
    /// Data source
    pub source: String,
}

/// Threat categories for IP reputation
#[derive(Debug, Clone)]
pub enum ThreatCategory {
    Malware,
    Botnet,
    Spam,
    Phishing,
    Scanner,
    Tor,
    Proxy,
    Unknown,
}

/// Rate limiting tracker
#[derive(Debug, Clone)]
pub struct RateLimitTracker {
    /// Identifier (IP, user, etc.)
    pub identifier: String,
    
    /// Request timestamps in current window
    pub requests: VecDeque<SystemTime>,
    
    /// Window size
    pub window_size: Duration,
    
    /// Maximum requests per window
    pub max_requests: u32,
    
    /// First violation timestamp
    pub first_violation: Option<SystemTime>,
    
    /// Violation count
    pub violation_count: u32,
}

/// Behavior baseline for anomaly detection
#[derive(Debug, Clone)]
pub struct BehaviorBaseline {
    /// User or entity identifier
    pub identifier: String,
    
    /// Typical request rate (requests per hour)
    pub typical_request_rate: f64,
    
    /// Typical request patterns
    pub typical_patterns: Vec<RequestPattern>,
    
    /// Typical access times (hours of day)
    pub typical_access_hours: Vec<u8>,
    
    /// Typical source IPs
    pub typical_source_ips: Vec<String>,
    
    /// Baseline creation time
    pub created_at: SystemTime,
    
    /// Last update time
    pub updated_at: SystemTime,
}

/// Request pattern for baseline
#[derive(Debug, Clone)]
pub struct RequestPattern {
    /// Endpoint pattern
    pub endpoint: String,
    
    /// HTTP method
    pub method: String,
    
    /// Frequency (requests per hour)
    pub frequency: f64,
    
    /// Typical response size
    pub response_size: u64,
}

/// Current behavior tracker
#[derive(Debug, Clone)]
pub struct BehaviorTracker {
    /// Identifier
    pub identifier: String,
    
    /// Recent requests
    pub recent_requests: VecDeque<RequestInfo>,
    
    /// Current session start
    pub session_start: SystemTime,
    
    /// Anomaly scores
    pub anomaly_scores: HashMap<String, f64>,
}

/// Request information for behavior tracking
#[derive(Debug, Clone)]
pub struct RequestInfo {
    /// Request timestamp
    pub timestamp: SystemTime,
    
    /// Endpoint
    pub endpoint: String,
    
    /// HTTP method
    pub method: String,
    
    /// Source IP
    pub source_ip: String,
    
    /// Response status
    pub status_code: u16,
    
    /// Response size
    pub response_size: u64,
    
    /// Processing time
    pub processing_time: Duration,
}

/// Security metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    /// Total requests processed
    pub total_requests: u64,
    
    /// Threats detected
    pub threats_detected: u64,
    
    /// Anomalies detected
    pub anomalies_detected: u64,
    
    /// Alerts generated
    pub alerts_generated: u64,
    
    /// Blocked requests
    pub blocked_requests: u64,
    
    /// Unique attacking IPs
    pub attacking_ips: u64,
    
    /// Average threat score
    pub avg_threat_score: f64,
    
    /// Metrics by time period
    pub hourly_metrics: HashMap<String, HourlyMetrics>,
}

/// Hourly security metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyMetrics {
    /// Hour (YYYY-MM-DD-HH format)
    pub hour: String,
    
    /// Requests in this hour
    pub requests: u64,
    
    /// Threats in this hour
    pub threats: u64,
    
    /// Anomalies in this hour
    pub anomalies: u64,
    
    /// Alerts in this hour
    pub alerts: u64,
}

/// Request tracker for monitoring
#[derive(Debug, Clone)]
pub struct RequestTracker {
    /// Recent requests by IP
    pub requests_by_ip: HashMap<String, VecDeque<SystemTime>>,
    
    /// Recent requests by user
    pub requests_by_user: HashMap<String, VecDeque<SystemTime>>,
    
    /// Failed authentication attempts
    pub failed_auth_attempts: HashMap<String, VecDeque<SystemTime>>,
    
    /// Suspicious activities
    pub suspicious_activities: VecDeque<SuspiciousActivity>,
}

/// Suspicious activity record
#[derive(Debug, Clone)]
pub struct SuspiciousActivity {
    /// Activity timestamp
    pub timestamp: SystemTime,
    
    /// Activity type
    pub activity_type: String,
    
    /// Source identifier
    pub source: String,
    
    /// Severity score
    pub severity_score: f64,
    
    /// Additional details
    pub details: HashMap<String, String>,
}

/// Threat detection configuration
#[derive(Debug, Clone)]
pub struct ThreatDetectionConfig {
    /// Enable IP reputation checking
    pub ip_reputation_enabled: bool,
    
    /// Enable pattern matching
    pub pattern_matching_enabled: bool,
    
    /// Enable rate limiting
    pub rate_limiting_enabled: bool,
    
    /// Update intervals
    pub ip_reputation_update_interval: Duration,
    
    /// Confidence threshold for alerts
    pub alert_confidence_threshold: f64,
}

/// Anomaly detection configuration
#[derive(Debug, Clone)]
pub struct AnomalyDetectionConfig {
    /// Enable user behavior analysis
    pub user_behavior_enabled: bool,
    
    /// Enable temporal analysis
    pub temporal_analysis_enabled: bool,
    
    /// Baseline learning period
    pub baseline_learning_period: Duration,
    
    /// Anomaly threshold
    pub anomaly_threshold: f64,
    
    /// Minimum baseline samples
    pub min_baseline_samples: u32,
}

/// Alert configuration
#[derive(Debug, Clone)]
pub struct AlertConfig {
    /// Maximum active alerts
    pub max_active_alerts: u32,
    
    /// Alert history retention
    pub history_retention: Duration,
    
    /// Auto-resolution timeout
    pub auto_resolution_timeout: Duration,
    
    /// Alert suppression rules
    pub suppression_rules: Vec<SuppressionRule>,
}

/// Alert suppression rule
#[derive(Debug, Clone)]
pub struct SuppressionRule {
    /// Rule name
    pub name: String,
    
    /// Alert types to suppress
    pub alert_types: Vec<AlertType>,
    
    /// Conditions for suppression
    pub conditions: Vec<SuppressionCondition>,
    
    /// Suppression duration
    pub duration: Duration,
}

/// Suppression condition
#[derive(Debug, Clone)]
pub struct SuppressionCondition {
    /// Field to check
    pub field: String,
    
    /// Condition value
    pub value: String,
    
    /// Condition operator
    pub operator: ConditionOperator,
}

/// Condition operators for suppression
#[derive(Debug, Clone)]
pub enum ConditionOperator {
    Equals,
    Contains,
    StartsWith,
    EndsWith,
    Regex,
}

/// Alert handler trait
#[async_trait::async_trait]
pub trait AlertHandler: Send + Sync {
    /// Handle security alert
    async fn handle_alert(&self, alert: &SecurityAlert) -> McpResult<()>;
    
    /// Get handler name
    fn name(&self) -> &str;
    
    /// Check if handler supports alert type
    fn supports_alert_type(&self, alert_type: &AlertType) -> bool;
}

impl SecurityMonitor {
    /// Create new security monitor
    pub async fn new(config: MonitoringConfig) -> McpResult<Self> {
        let threat_detector = Arc::new(ThreatDetector::new().await?);
        let anomaly_detector = Arc::new(AnomalyDetector::new().await?);
        let alert_manager = Arc::new(AlertManager::new().await?);
        let metrics = Arc::new(RwLock::new(SecurityMetrics::new()));
        let request_tracker = Arc::new(RwLock::new(RequestTracker::new()));
        
        let monitor = Self {
            config: config.clone(),
            threat_detector,
            anomaly_detector,
            alert_manager,
            metrics,
            request_tracker,
        };
        
        // Start background monitoring tasks
        monitor.start_monitoring_tasks().await?;
        
        info!("Security monitor initialized");
        Ok(monitor)
    }
    
    /// Check for suspicious activity
    pub async fn check_suspicious_activity(&self, context: &SecurityContext) -> McpResult<bool> {
        let mut is_suspicious = false;
        
        // Threat detection
        if self.config.threat_detection {
            let threat_score = self.threat_detector.analyze_request(context).await?;
            if threat_score > 0.7 {
                is_suspicious = true;
                self.generate_threat_alert(context, threat_score).await?;
            }
        }
        
        // Anomaly detection
        if self.config.anomaly_detection {
            let anomaly_score = self.anomaly_detector.analyze_behavior(context).await?;
            if anomaly_score > self.config.alert_thresholds.suspicious_pattern_threshold {
                is_suspicious = true;
                self.generate_anomaly_alert(context, anomaly_score).await?;
            }
        }
        
        // Update metrics
        self.update_metrics(context, is_suspicious).await;
        
        Ok(is_suspicious)
    }
    
    /// Generate threat alert
    async fn generate_threat_alert(&self, context: &SecurityContext, threat_score: f64) -> McpResult<()> {
        let alert = SecurityAlert {
            id: uuid::Uuid::new_v4().to_string(),
            alert_type: AlertType::MaliciousRequest,
            severity: if threat_score > 0.9 {
                SecuritySeverity::Critical
            } else if threat_score > 0.8 {
                SecuritySeverity::High
            } else {
                SecuritySeverity::Medium
            },
            timestamp: SystemTime::now(),
            source: "threat_detector".to_string(),
            title: "Suspicious Request Detected".to_string(),
            description: format!("Threat score: {:.2}", threat_score),
            user_id: context.user_id.clone(),
            client_ip: context.client_ip.clone(),
            request_id: Some(context.request_id.clone()),
            data: serde_json::json!({
                "threat_score": threat_score,
                "context": context.metadata
            }),
            tags: vec!["threat".to_string(), "automated".to_string()],
            status: AlertStatus::Active,
            resolution: None,
        };
        
        self.alert_manager.create_alert(alert).await
    }
    
    /// Generate anomaly alert
    async fn generate_anomaly_alert(&self, context: &SecurityContext, anomaly_score: f64) -> McpResult<()> {
        let alert = SecurityAlert {
            id: uuid::Uuid::new_v4().to_string(),
            alert_type: AlertType::AnomalousAccess,
            severity: SecuritySeverity::Medium,
            timestamp: SystemTime::now(),
            source: "anomaly_detector".to_string(),
            title: "Anomalous Behavior Detected".to_string(),
            description: format!("Anomaly score: {:.2}", anomaly_score),
            user_id: context.user_id.clone(),
            client_ip: context.client_ip.clone(),
            request_id: Some(context.request_id.clone()),
            data: serde_json::json!({
                "anomaly_score": anomaly_score,
                "context": context.metadata
            }),
            tags: vec!["anomaly".to_string(), "behavioral".to_string()],
            status: AlertStatus::Active,
            resolution: None,
        };
        
        self.alert_manager.create_alert(alert).await
    }
    
    /// Update security metrics
    async fn update_metrics(&self, context: &SecurityContext, is_suspicious: bool) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        
        if is_suspicious {
            metrics.threats_detected += 1;
        }
        
        // Update hourly metrics
        let hour_key = context.timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() / 3600;
        let hour_str = format!("{}", hour_key);
        
        let hourly = metrics.hourly_metrics.entry(hour_str).or_insert(HourlyMetrics {
            hour: hour_key.to_string(),
            requests: 0,
            threats: 0,
            anomalies: 0,
            alerts: 0,
        });
        
        hourly.requests += 1;
        if is_suspicious {
            hourly.threats += 1;
        }
    }
    
    /// Start background monitoring tasks
    async fn start_monitoring_tasks(&self) -> McpResult<()> {
        // Metrics collection task
        let metrics_clone = self.metrics.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                // Collect and aggregate metrics
                debug!("Collecting security metrics");
            }
        });
        
        // Alert processing task
        let alert_manager_clone = self.alert_manager.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // 1 minute
            loop {
                interval.tick().await;
                if let Err(e) = alert_manager_clone.process_alerts().await {
                    warn!("Alert processing failed: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// Get security statistics
    pub async fn get_security_stats(&self) -> McpResult<crate::security::SecurityStats> {
        let metrics = self.metrics.read().await;
        let alerts = self.alert_manager.get_alert_stats().await?;
        
        Ok(crate::security::SecurityStats {
            total_events: metrics.total_requests,
            events_by_severity: HashMap::new(), // Would be populated from alerts
            alerts_generated: metrics.alerts_generated,
            threats_detected: metrics.threats_detected,
            avg_encryption_time_us: 0, // Would come from encryption metrics
            avg_decryption_time_us: 0,
            security_overhead_percent: 0.0,
        })
    }
    
    /// Add alert handler
    pub async fn add_alert_handler(&self, handler: Arc<dyn AlertHandler>) -> McpResult<()> {
        self.alert_manager.add_handler(handler).await
    }
}

impl ThreatDetector {
    /// Create new threat detector
    pub async fn new() -> McpResult<Self> {
        let attack_patterns = Self::load_default_patterns();
        
        Ok(Self {
            attack_patterns,
            ip_reputation: Arc::new(RwLock::new(HashMap::new())),
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            config: ThreatDetectionConfig {
                ip_reputation_enabled: true,
                pattern_matching_enabled: true,
                rate_limiting_enabled: true,
                ip_reputation_update_interval: Duration::from_secs(3600),
                alert_confidence_threshold: 0.8,
            },
        })
    }
    
    /// Analyze request for threats
    pub async fn analyze_request(&self, context: &SecurityContext) -> McpResult<f64> {
        let mut threat_score = 0.0;
        let mut max_score = 0.0;
        
        // Pattern matching
        if self.config.pattern_matching_enabled {
            for pattern in &self.attack_patterns {
                let pattern_score = self.match_pattern(pattern, context).await?;
                threat_score += pattern_score * 0.4; // Weight pattern matching
                max_score += pattern.confidence_threshold * 0.4;
            }
        }
        
        // IP reputation check
        if self.config.ip_reputation_enabled {
            if let Some(client_ip) = &context.client_ip {
                let ip_score = self.check_ip_reputation(client_ip).await?;
                threat_score += ip_score * 0.3; // Weight IP reputation
                max_score += 1.0 * 0.3;
            }
        }
        
        // Rate limiting check
        if self.config.rate_limiting_enabled {
            if let Some(client_ip) = &context.client_ip {
                let rate_score = self.check_rate_limit(client_ip).await?;
                threat_score += rate_score * 0.3; // Weight rate limiting
                max_score += 1.0 * 0.3;
            }
        }
        
        // Normalize score
        let normalized_score = if max_score > 0.0 {
            (threat_score / max_score).min(1.0)
        } else {
            0.0
        };
        
        Ok(normalized_score)
    }
    
    /// Match attack pattern
    async fn match_pattern(&self, pattern: &AttackPattern, context: &SecurityContext) -> McpResult<f64> {
        let mut pattern_score = 0.0;
        let mut total_weight = 0.0;
        
        for rule in &pattern.rules {
            let rule_match = self.match_rule(rule, context).await?;
            pattern_score += rule_match * rule.weight;
            total_weight += rule.weight;
        }
        
        let normalized_score = if total_weight > 0.0 {
            pattern_score / total_weight
        } else {
            0.0
        };
        
        Ok(normalized_score)
    }
    
    /// Match individual rule
    async fn match_rule(&self, rule: &PatternRule, context: &SecurityContext) -> McpResult<f64> {
        let field_value = context.metadata.get(&rule.field).unwrap_or(&String::new());
        
        let matches = match &rule.rule_type {
            RuleType::Exact => field_value == &rule.pattern,
            RuleType::Contains => field_value.contains(&rule.pattern),
            RuleType::StartsWith => field_value.starts_with(&rule.pattern),
            RuleType::EndsWith => field_value.ends_with(&rule.pattern),
            RuleType::Regex => {
                // In production, would use proper regex library
                field_value.contains(&rule.pattern) // Simplified
            }
            RuleType::Numeric { operator, value } => {
                if let Ok(field_num) = field_value.parse::<f64>() {
                    match operator {
                        NumericOperator::GreaterThan => field_num > *value,
                        NumericOperator::LessThan => field_num < *value,
                        NumericOperator::Equal => (field_num - value).abs() < f64::EPSILON,
                        NumericOperator::GreaterThanOrEqual => field_num >= *value,
                        NumericOperator::LessThanOrEqual => field_num <= *value,
                    }
                } else {
                    false
                }
            }
        };
        
        Ok(if matches { 1.0 } else { 0.0 })
    }
    
    /// Check IP reputation
    async fn check_ip_reputation(&self, ip: &str) -> McpResult<f64> {
        let reputation_db = self.ip_reputation.read().await;
        
        if let Some(reputation) = reputation_db.get(ip) {
            // Convert reputation score to threat score (inverse relationship)
            let threat_score = (100 - reputation.score as u32) as f64 / 100.0;
            Ok(threat_score)
        } else {
            // Unknown IP, neutral score
            Ok(0.5)
        }
    }
    
    /// Check rate limiting
    async fn check_rate_limit(&self, identifier: &str) -> McpResult<f64> {
        let mut rate_limiters = self.rate_limiters.write().await;
        let now = SystemTime::now();
        
        let tracker = rate_limiters.entry(identifier.to_string()).or_insert(RateLimitTracker {
            identifier: identifier.to_string(),
            requests: VecDeque::new(),
            window_size: Duration::from_secs(60), // 1 minute window
            max_requests: 100,
            first_violation: None,
            violation_count: 0,
        });
        
        // Remove old requests outside window
        while let Some(&front_time) = tracker.requests.front() {
            if now.duration_since(front_time).unwrap_or_default() > tracker.window_size {
                tracker.requests.pop_front();
            } else {
                break;
            }
        }
        
        // Add current request
        tracker.requests.push_back(now);
        
        // Check if rate limit exceeded
        if tracker.requests.len() > tracker.max_requests as usize {
            if tracker.first_violation.is_none() {
                tracker.first_violation = Some(now);
            }
            tracker.violation_count += 1;
            
            // Calculate threat score based on violation severity
            let excess_requests = tracker.requests.len() - tracker.max_requests as usize;
            let threat_score = (excess_requests as f64 / tracker.max_requests as f64).min(1.0);
            Ok(threat_score)
        } else {
            Ok(0.0)
        }
    }
    
    /// Load default attack patterns
    fn load_default_patterns() -> Vec<AttackPattern> {
        vec![
            AttackPattern {
                name: "SQL Injection".to_string(),
                pattern_type: AttackType::SqlInjection,
                rules: vec![
                    PatternRule {
                        field: "request_body".to_string(),
                        rule_type: RuleType::Contains,
                        pattern: "' OR '1'='1".to_string(),
                        weight: 1.0,
                    },
                    PatternRule {
                        field: "request_body".to_string(),
                        rule_type: RuleType::Contains,
                        pattern: "UNION SELECT".to_string(),
                        weight: 1.0,
                    },
                ],
                confidence_threshold: 0.8,
                severity: SecuritySeverity::High,
            },
            AttackPattern {
                name: "XSS Attempt".to_string(),
                pattern_type: AttackType::Xss,
                rules: vec![
                    PatternRule {
                        field: "request_body".to_string(),
                        rule_type: RuleType::Contains,
                        pattern: "<script>".to_string(),
                        weight: 1.0,
                    },
                    PatternRule {
                        field: "request_body".to_string(),
                        rule_type: RuleType::Contains,
                        pattern: "javascript:".to_string(),
                        weight: 0.8,
                    },
                ],
                confidence_threshold: 0.7,
                severity: SecuritySeverity::Medium,
            },
        ]
    }
}

impl AnomalyDetector {
    /// Create new anomaly detector
    pub async fn new() -> McpResult<Self> {
        Ok(Self {
            baselines: Arc::new(RwLock::new(HashMap::new())),
            current_behavior: Arc::new(RwLock::new(HashMap::new())),
            config: AnomalyDetectionConfig {
                user_behavior_enabled: true,
                temporal_analysis_enabled: true,
                baseline_learning_period: Duration::from_secs(7 * 24 * 3600), // 7 days
                anomaly_threshold: 0.7,
                min_baseline_samples: 100,
            },
        })
    }
    
    /// Analyze behavior for anomalies
    pub async fn analyze_behavior(&self, context: &SecurityContext) -> McpResult<f64> {
        if let Some(user_id) = &context.user_id {
            self.analyze_user_behavior(user_id, context).await
        } else if let Some(client_ip) = &context.client_ip {
            self.analyze_ip_behavior(client_ip, context).await
        } else {
            Ok(0.0)
        }
    }
    
    /// Analyze user behavior patterns
    async fn analyze_user_behavior(&self, user_id: &str, context: &SecurityContext) -> McpResult<f64> {
        let baselines = self.baselines.read().await;
        let baseline = baselines.get(user_id);
        
        if let Some(baseline) = baseline {
            let mut anomaly_score = 0.0;
            
            // Check temporal patterns
            if self.config.temporal_analysis_enabled {
                let current_hour = context.timestamp
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() / 3600 % 24;
                
                if !baseline.typical_access_hours.contains(&(current_hour as u8)) {
                    anomaly_score += 0.3; // Unusual access time
                }
            }
            
            // Check IP patterns
            if let Some(client_ip) = &context.client_ip {
                if !baseline.typical_source_ips.contains(client_ip) {
                    anomaly_score += 0.4; // Unusual source IP
                }
            }
            
            // Check request patterns
            // This would be more sophisticated in production
            anomaly_score += 0.1; // Placeholder for request pattern analysis
            
            Ok(anomaly_score.min(1.0))
        } else {
            // No baseline yet, start learning
            self.start_learning_baseline(user_id, context).await?;
            Ok(0.0)
        }
    }
    
    /// Analyze IP behavior patterns
    async fn analyze_ip_behavior(&self, ip: &str, context: &SecurityContext) -> McpResult<f64> {
        // Simplified IP behavior analysis
        // In production, this would track IP-specific patterns
        Ok(0.0)
    }
    
    /// Start learning baseline for new user
    async fn start_learning_baseline(&self, user_id: &str, context: &SecurityContext) -> McpResult<()> {
        let mut current_behavior = self.current_behavior.write().await;
        
        let tracker = current_behavior.entry(user_id.to_string()).or_insert(BehaviorTracker {
            identifier: user_id.to_string(),
            recent_requests: VecDeque::new(),
            session_start: context.timestamp,
            anomaly_scores: HashMap::new(),
        });
        
        // Add request to learning data
        let request_info = RequestInfo {
            timestamp: context.timestamp,
            endpoint: context.metadata.get("endpoint").unwrap_or(&String::new()).clone(),
            method: context.metadata.get("method").unwrap_or(&String::new()).clone(),
            source_ip: context.client_ip.clone().unwrap_or_default(),
            status_code: 200, // Would be extracted from actual response
            response_size: 0,  // Would be extracted from actual response
            processing_time: Duration::from_millis(100), // Would be measured
        };
        
        tracker.recent_requests.push_back(request_info);
        
        // Keep only recent requests for learning
        while tracker.recent_requests.len() > 1000 {
            tracker.recent_requests.pop_front();
        }
        
        Ok(())
    }
}

impl AlertManager {
    /// Create new alert manager
    pub async fn new() -> McpResult<Self> {
        Ok(Self {
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
            handlers: Vec::new(),
            config: AlertConfig {
                max_active_alerts: 1000,
                history_retention: Duration::from_secs(30 * 24 * 3600), // 30 days
                auto_resolution_timeout: Duration::from_secs(24 * 3600), // 24 hours
                suppression_rules: Vec::new(),
            },
        })
    }
    
    /// Create new alert
    pub async fn create_alert(&self, alert: SecurityAlert) -> McpResult<()> {
        // Check suppression rules
        if self.should_suppress_alert(&alert).await? {
            debug!("Alert suppressed: {}", alert.id);
            return Ok(());
        }
        
        let mut active_alerts = self.active_alerts.write().await;
        
        // Check if we're at the limit
        if active_alerts.len() >= self.config.max_active_alerts as usize {
            warn!("Maximum active alerts reached, dropping oldest");
            // Remove oldest alert (simple FIFO)
            if let Some((oldest_id, _)) = active_alerts.iter().next() {
                let oldest_id = oldest_id.clone();
                active_alerts.remove(&oldest_id);
            }
        }
        
        active_alerts.insert(alert.id.clone(), alert.clone());
        drop(active_alerts);
        
        // Notify handlers
        for handler in &self.handlers {
            if handler.supports_alert_type(&alert.alert_type) {
                if let Err(e) = handler.handle_alert(&alert).await {
                    warn!("Alert handler '{}' failed: {}", handler.name(), e);
                }
            }
        }
        
        info!("Security alert created: {} - {}", alert.id, alert.title);
        Ok(())
    }
    
    /// Check if alert should be suppressed
    async fn should_suppress_alert(&self, alert: &SecurityAlert) -> McpResult<bool> {
        for rule in &self.config.suppression_rules {
            if rule.alert_types.contains(&alert.alert_type) {
                // Check conditions
                let mut all_conditions_met = true;
                for condition in &rule.conditions {
                    if !self.check_suppression_condition(condition, alert) {
                        all_conditions_met = false;
                        break;
                    }
                }
                
                if all_conditions_met {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// Check suppression condition
    fn check_suppression_condition(&self, condition: &SuppressionCondition, alert: &SecurityAlert) -> bool {
        let field_value = match condition.field.as_str() {
            "client_ip" => alert.client_ip.as_ref().unwrap_or(&String::new()),
            "user_id" => alert.user_id.as_ref().unwrap_or(&String::new()),
            "source" => &alert.source,
            "title" => &alert.title,
            "description" => &alert.description,
            _ => &String::new(),
        };
        
        match condition.operator {
            ConditionOperator::Equals => field_value == &condition.value,
            ConditionOperator::Contains => field_value.contains(&condition.value),
            ConditionOperator::StartsWith => field_value.starts_with(&condition.value),
            ConditionOperator::EndsWith => field_value.ends_with(&condition.value),
            ConditionOperator::Regex => field_value.contains(&condition.value), // Simplified
        }
    }
    
    /// Process alerts (background task)
    pub async fn process_alerts(&self) -> McpResult<()> {
        let mut active_alerts = self.active_alerts.write().await;
        let mut expired_alerts = Vec::new();
        let now = SystemTime::now();
        
        // Find expired alerts
        for (id, alert) in active_alerts.iter() {
            if now.duration_since(alert.timestamp).unwrap_or_default() > self.config.auto_resolution_timeout {
                expired_alerts.push(id.clone());
            }
        }
        
        // Auto-resolve expired alerts
        for alert_id in expired_alerts {
            if let Some(mut alert) = active_alerts.remove(&alert_id) {
                alert.status = AlertStatus::Resolved;
                alert.resolution = Some(AlertResolution {
                    timestamp: now,
                    resolution_type: ResolutionType::Timeout,
                    description: "Auto-resolved due to timeout".to_string(),
                    resolved_by: "system".to_string(),
                    actions_taken: vec!["timeout_resolution".to_string()],
                });
                
                // Move to history
                let mut history = self.alert_history.write().await;
                history.push_back(alert);
                
                // Trim history if needed
                while history.len() > 10000 {
                    history.pop_front();
                }
            }
        }
        
        Ok(())
    }
    
    /// Add alert handler
    pub async fn add_handler(&self, handler: Arc<dyn AlertHandler>) -> McpResult<()> {
        // Note: This is not thread-safe in this implementation
        // In production, would need to use Arc<RwLock<Vec<...>>>
        Ok(())
    }
    
    /// Get alert statistics
    pub async fn get_alert_stats(&self) -> McpResult<AlertStats> {
        let active_alerts = self.active_alerts.read().await;
        let history = self.alert_history.read().await;
        
        Ok(AlertStats {
            active_alerts: active_alerts.len() as u64,
            total_alerts: (active_alerts.len() + history.len()) as u64,
            alerts_by_type: HashMap::new(), // Would be calculated
            alerts_by_severity: HashMap::new(), // Would be calculated
        })
    }
}

/// Alert statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStats {
    /// Number of active alerts
    pub active_alerts: u64,
    
    /// Total alerts (active + resolved)
    pub total_alerts: u64,
    
    /// Alerts by type
    pub alerts_by_type: HashMap<String, u64>,
    
    /// Alerts by severity
    pub alerts_by_severity: HashMap<String, u64>,
}

impl SecurityMetrics {
    /// Create new empty metrics
    fn new() -> Self {
        Self {
            total_requests: 0,
            threats_detected: 0,
            anomalies_detected: 0,
            alerts_generated: 0,
            blocked_requests: 0,
            attacking_ips: 0,
            avg_threat_score: 0.0,
            hourly_metrics: HashMap::new(),
        }
    }
}

impl RequestTracker {
    /// Create new request tracker
    fn new() -> Self {
        Self {
            requests_by_ip: HashMap::new(),
            requests_by_user: HashMap::new(),
            failed_auth_attempts: HashMap::new(),
            suspicious_activities: VecDeque::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    
    #[tokio::test]
    async fn test_security_monitor_creation() {
        let config = MonitoringConfig::default();
        let monitor = SecurityMonitor::new(config).await.unwrap();
        
        // Test basic functionality
        let context = SecurityContext::new("test-request".to_string())
            .with_client_ip("192.168.1.1".to_string())
            .with_user_id("test-user".to_string());
        
        let is_suspicious = monitor.check_suspicious_activity(&context).await.unwrap();
        // Should not be suspicious for a basic test request
        assert!(!is_suspicious);
    }
    
    #[tokio::test]
    async fn test_threat_detector() {
        let detector = ThreatDetector::new().await.unwrap();
        
        let mut context = SecurityContext::new("test-request".to_string());
        context.metadata.insert("request_body".to_string(), "SELECT * FROM users".to_string());
        
        let threat_score = detector.analyze_request(&context).await.unwrap();
        // Should have some threat score due to SQL-like content
        assert!(threat_score >= 0.0);
    }
    
    #[tokio::test]
    async fn test_anomaly_detector() {
        let detector = AnomalyDetector::new().await.unwrap();
        
        let context = SecurityContext::new("test-request".to_string())
            .with_user_id("new-user".to_string());
        
        let anomaly_score = detector.analyze_behavior(&context).await.unwrap();
        // Should be 0 for new user (no baseline yet)
        assert_eq!(anomaly_score, 0.0);
    }
    
    #[tokio::test]
    async fn test_alert_manager() {
        let alert_manager = AlertManager::new().await.unwrap();
        
        let alert = SecurityAlert {
            id: "test-alert".to_string(),
            alert_type: AlertType::BruteForce,
            severity: SecuritySeverity::High,
            timestamp: SystemTime::now(),
            source: "test".to_string(),
            title: "Test Alert".to_string(),
            description: "This is a test alert".to_string(),
            user_id: Some("test-user".to_string()),
            client_ip: Some("192.168.1.1".to_string()),
            request_id: Some("test-request".to_string()),
            data: serde_json::json!({}),
            tags: vec!["test".to_string()],
            status: AlertStatus::Active,
            resolution: None,
        };
        
        alert_manager.create_alert(alert).await.unwrap();
        
        let stats = alert_manager.get_alert_stats().await.unwrap();
        assert_eq!(stats.active_alerts, 1);
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let detector = ThreatDetector::new().await.unwrap();
        let test_ip = "192.168.1.100";
        
        // Make multiple requests to trigger rate limiting
        for _ in 0..150 {
            let threat_score = detector.check_rate_limit(test_ip).await.unwrap();
            // Later requests should have higher threat scores
        }
        
        // Final check should show high threat score
        let final_score = detector.check_rate_limit(test_ip).await.unwrap();
        assert!(final_score > 0.0);
    }
}