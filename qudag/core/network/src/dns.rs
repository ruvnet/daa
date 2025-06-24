#![deny(unsafe_code)]
#![warn(missing_docs)]

//! DNS integration module for ruv.io using Cloudflare API.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during DNS operations
#[derive(Error, Debug)]
pub enum DnsError {
    /// API request failed
    #[error("API request failed: {0}")]
    ApiError(String),

    /// Invalid record data
    #[error("Invalid record data: {0}")]
    ValidationError(String),

    /// Record not found
    #[error("Record not found: {0}")]
    NotFound(String),
}

/// DNS record types supported by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordType {
    /// A record pointing to IPv4 address
    A,
    /// AAAA record pointing to IPv6 address  
    AAAA,
    /// TXT record for storing text data
    TXT,
    /// CNAME record for aliases
    CNAME,
}

/// A DNS record entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    /// Record name/hostname
    pub name: String,
    /// Record type
    pub record_type: RecordType,
    /// Record content/value
    pub content: String,
    /// Time-to-live in seconds
    pub ttl: u32,
    /// Proxied through Cloudflare
    pub proxied: bool,
}

/// Configuration for Cloudflare API client
#[derive(Debug, Clone)]
pub struct CloudflareConfig {
    /// API token for authentication
    api_token: String,
    /// Zone ID for the domain
    zone_id: String,
}

/// Client for interacting with Cloudflare DNS API
pub struct CloudflareClient {
    config: CloudflareConfig,
    http_client: reqwest::Client,
}

impl CloudflareClient {
    /// Creates a new Cloudflare API client
    pub fn new(config: CloudflareConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    const API_BASE: &'static str = "https://api.cloudflare.com/client/v4";

    /// Lists all DNS records in the zone
    pub async fn list_records(&self) -> Result<Vec<DnsRecord>, DnsError> {
        let url = format!(
            "{}/zones/{}/dns_records",
            Self::API_BASE,
            self.config.zone_id
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| DnsError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DnsError::ApiError(format!(
                "API request failed: {}",
                response.status()
            )));
        }

        let records = response
            .json::<Vec<DnsRecord>>()
            .await
            .map_err(|e| DnsError::ApiError(e.to_string()))?;

        Ok(records)
    }

    /// Creates a new DNS record
    pub async fn create_record(&self, record: DnsRecord) -> Result<DnsRecord, DnsError> {
        let url = format!(
            "{}/zones/{}/dns_records",
            Self::API_BASE,
            self.config.zone_id
        );

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .header("Content-Type", "application/json")
            .json(&record)
            .send()
            .await
            .map_err(|e| DnsError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DnsError::ApiError(format!(
                "API request failed: {}",
                response.status()
            )));
        }

        let created_record = response
            .json::<DnsRecord>()
            .await
            .map_err(|e| DnsError::ApiError(e.to_string()))?;

        Ok(created_record)
    }

    /// Updates an existing DNS record
    pub async fn update_record(
        &self,
        record_id: &str,
        record: DnsRecord,
    ) -> Result<DnsRecord, DnsError> {
        let url = format!(
            "{}/zones/{}/dns_records/{}",
            Self::API_BASE,
            self.config.zone_id,
            record_id
        );

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .header("Content-Type", "application/json")
            .json(&record)
            .send()
            .await
            .map_err(|e| DnsError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DnsError::ApiError(format!(
                "API request failed: {}",
                response.status()
            )));
        }

        let updated_record = response
            .json::<DnsRecord>()
            .await
            .map_err(|e| DnsError::ApiError(e.to_string()))?;

        Ok(updated_record)
    }

    /// Deletes a DNS record
    pub async fn delete_record(&self, record_id: &str) -> Result<(), DnsError> {
        let url = format!(
            "{}/zones/{}/dns_records/{}",
            Self::API_BASE,
            self.config.zone_id,
            record_id
        );

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| DnsError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DnsError::ApiError(format!(
                "API request failed: {}",
                response.status()
            )));
        }

        Ok(())
    }
}

/// DNS record manager for handling record operations
pub struct DnsManager {
    client: CloudflareClient,
}

impl DnsManager {
    /// Creates a new DNS record manager
    pub fn new(config: CloudflareConfig) -> Self {
        Self {
            client: CloudflareClient::new(config),
        }
    }

    /// Lists all DNS records
    pub async fn list_records(&self) -> Result<Vec<DnsRecord>, DnsError> {
        self.client.list_records().await
    }

    /// Creates a new DNS record
    pub async fn create_record(&self, record: DnsRecord) -> Result<DnsRecord, DnsError> {
        // Validate record data
        self.validate_record(&record)?;
        self.client.create_record(record).await
    }

    /// Updates an existing DNS record
    pub async fn update_record(
        &self,
        record_id: &str,
        record: DnsRecord,
    ) -> Result<DnsRecord, DnsError> {
        // Validate record data
        self.validate_record(&record)?;
        self.client.update_record(record_id, record).await
    }

    /// Deletes a DNS record
    pub async fn delete_record(&self, record_id: &str) -> Result<(), DnsError> {
        self.client.delete_record(record_id).await
    }

    /// Validates record data before operations
    fn validate_record(&self, record: &DnsRecord) -> Result<(), DnsError> {
        // Validate record name
        if record.name.is_empty() || record.name.len() > 255 {
            return Err(DnsError::ValidationError(
                "Invalid record name length".to_string(),
            ));
        }

        // Basic hostname validation
        if !record
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        {
            return Err(DnsError::ValidationError(
                "Invalid characters in record name".to_string(),
            ));
        }

        // Validate content based on record type
        match record.record_type {
            RecordType::A => {
                // Validate IPv4 address
                if !record.content.split('.').count() == 4
                    && !record
                        .content
                        .split('.')
                        .all(|octet| octet.parse::<u8>().is_ok())
                {
                    return Err(DnsError::ValidationError(
                        "Invalid IPv4 address".to_string(),
                    ));
                }
            }
            RecordType::AAAA => {
                // Basic IPv6 validation
                if !record.content.contains(':') || record.content.len() > 39 {
                    return Err(DnsError::ValidationError(
                        "Invalid IPv6 address".to_string(),
                    ));
                }
            }
            RecordType::TXT => {
                // Validate TXT record length
                if record.content.len() > 255 {
                    return Err(DnsError::ValidationError("TXT record too long".to_string()));
                }
            }
            RecordType::CNAME => {
                // Validate CNAME is a valid hostname
                if !record
                    .content
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
                {
                    return Err(DnsError::ValidationError("Invalid CNAME value".to_string()));
                }
            }
        }

        // Validate TTL
        if record.ttl < 60 || record.ttl > 86400 {
            return Err(DnsError::ValidationError(
                "TTL must be between 60 and 86400 seconds".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;
    use serde_json::json;

    fn setup_test_config() -> CloudflareConfig {
        CloudflareConfig {
            api_token: "test_token".to_string(),
            zone_id: "test_zone".to_string(),
        }
    }

    fn create_test_record() -> DnsRecord {
        DnsRecord {
            name: "test.example.com".to_string(),
            record_type: RecordType::A,
            content: "192.0.2.1".to_string(),
            ttl: 3600,
            proxied: false,
        }
    }

    #[tokio::test]
    async fn test_list_records() {
        let _m = mock("GET", "/zones/test_zone/dns_records")
            .with_header("Authorization", "Bearer test_token")
            .with_status(200)
            .with_body(
                json!({
                    "success": true,
                    "result": [{
                        "name": "test.example.com",
                        "type": "A",
                        "content": "192.0.2.1",
                        "ttl": 3600,
                        "proxied": false
                    }]
                })
                .to_string(),
            )
            .create();

        let client = CloudflareClient::new(setup_test_config());
        let records = client.list_records().await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].name, "test.example.com");
    }

    #[tokio::test]
    async fn test_create_record() {
        let record = create_test_record();
        let _m = mock("POST", "/zones/test_zone/dns_records")
            .with_header("Authorization", "Bearer test_token")
            .with_status(200)
            .with_body(
                json!({
                    "success": true,
                    "result": {
                        "name": "test.example.com",
                        "type": "A",
                        "content": "192.0.2.1",
                        "ttl": 3600,
                        "proxied": false
                    }
                })
                .to_string(),
            )
            .create();

        let client = CloudflareClient::new(setup_test_config());
        let created = client.create_record(record.clone()).await.unwrap();
        assert_eq!(created.name, record.name);
        assert_eq!(created.content, record.content);
    }

    #[tokio::test]
    async fn test_update_record() {
        let record = create_test_record();
        let _m = mock("PUT", "/zones/test_zone/dns_records/test_id")
            .with_header("Authorization", "Bearer test_token")
            .with_status(200)
            .with_body(
                json!({
                    "success": true,
                    "result": {
                        "name": "test.example.com",
                        "type": "A",
                        "content": "192.0.2.2",
                        "ttl": 3600,
                        "proxied": false
                    }
                })
                .to_string(),
            )
            .create();

        let client = CloudflareClient::new(setup_test_config());
        let updated = client.update_record("test_id", record).await.unwrap();
        assert_eq!(updated.content, "192.0.2.2");
    }

    #[tokio::test]
    async fn test_delete_record() {
        let _m = mock("DELETE", "/zones/test_zone/dns_records/test_id")
            .with_header("Authorization", "Bearer test_token")
            .with_status(200)
            .with_body(
                json!({
                    "success": true,
                    "result": {}
                })
                .to_string(),
            )
            .create();

        let client = CloudflareClient::new(setup_test_config());
        client.delete_record("test_id").await.unwrap();
    }

    #[test]
    fn test_record_validation() {
        let dns_manager = DnsManager::new(setup_test_config());

        // Test valid record
        let valid_record = create_test_record();
        assert!(dns_manager.validate_record(&valid_record).is_ok());

        // Test invalid name
        let mut invalid_record = valid_record.clone();
        invalid_record.name = "invalid@name".to_string();
        assert!(dns_manager.validate_record(&invalid_record).is_err());

        // Test invalid IPv4
        let mut invalid_record = valid_record.clone();
        invalid_record.content = "256.256.256.256".to_string();
        assert!(dns_manager.validate_record(&invalid_record).is_err());

        // Test invalid TTL
        let mut invalid_record = valid_record.clone();
        invalid_record.ttl = 30; // Too low
        assert!(dns_manager.validate_record(&invalid_record).is_err());
    }
}
