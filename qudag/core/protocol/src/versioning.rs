//! Protocol versioning system for backward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use thiserror::Error;

use crate::message::{Message, MessageType, ProtocolVersion};

/// Type alias for custom migration function
pub type CustomMigrationFn = Box<dyn Fn(&Message) -> Result<Message, VersionError> + Send + Sync>;

/// Versioning-related errors
#[derive(Debug, Error)]
pub enum VersionError {
    /// Unsupported protocol version
    #[error("Unsupported protocol version: {version:?}")]
    UnsupportedVersion { version: ProtocolVersion },

    /// Incompatible protocol versions
    #[error("Incompatible protocol versions: {local:?} vs {remote:?}")]
    IncompatibleVersions {
        local: ProtocolVersion,
        remote: ProtocolVersion,
    },

    /// Feature not available in version
    #[error("Feature '{feature}' not available in version {version:?}")]
    FeatureNotAvailable {
        feature: String,
        version: ProtocolVersion,
    },

    /// Migration failed
    #[error("Migration from {from:?} to {to:?} failed: {reason}")]
    MigrationFailed {
        from: ProtocolVersion,
        to: ProtocolVersion,
        reason: String,
    },

    /// Serialization/deserialization error
    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },
}

/// Protocol version registry
#[derive(Debug, Clone)]
pub struct VersionRegistry {
    /// Supported versions
    supported_versions: Vec<VersionInfo>,
    /// Feature compatibility matrix
    feature_matrix: HashMap<ProtocolVersion, HashSet<String>>,
    /// Message type compatibility
    message_compatibility: HashMap<ProtocolVersion, HashSet<MessageType>>,
    /// Migration paths between versions
    migration_paths: HashMap<(ProtocolVersion, ProtocolVersion), MigrationStrategy>,
}

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Protocol version
    pub version: ProtocolVersion,
    /// Version name/codename
    pub name: String,
    /// Release date (ISO 8601 format)
    pub release_date: String,
    /// Supported features
    pub features: Vec<String>,
    /// Deprecated features
    pub deprecated_features: Vec<String>,
    /// Security requirements
    pub security_requirements: SecurityRequirements,
    /// Backward compatibility information
    pub compatibility: CompatibilityInfo,
}

/// Security requirements for a protocol version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// Minimum key sizes
    pub min_key_sizes: HashMap<String, u32>,
    /// Required cryptographic algorithms
    pub required_algorithms: Vec<String>,
    /// Forbidden algorithms (deprecated/insecure)
    pub forbidden_algorithms: Vec<String>,
    /// Quantum resistance requirements
    pub quantum_resistant: bool,
}

/// Compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    /// Versions this version is compatible with
    pub compatible_with: Vec<ProtocolVersion>,
    /// Minimum supported version for communication
    pub min_supported_version: ProtocolVersion,
    /// Breaking changes from previous version
    pub breaking_changes: Vec<String>,
    /// Migration notes
    pub migration_notes: Vec<String>,
}

/// Migration strategy between versions
pub enum MigrationStrategy {
    /// Direct migration (no transformation needed)
    Direct,
    /// Transform messages using converter function
    Transform(fn(&Message) -> Result<Message, VersionError>),
    /// Custom migration logic
    Custom(CustomMigrationFn),
    /// Not supported
    NotSupported,
}

impl std::fmt::Debug for MigrationStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationStrategy::Direct => write!(f, "Direct"),
            MigrationStrategy::Transform(_) => write!(f, "Transform(<function>)"),
            MigrationStrategy::Custom(_) => write!(f, "Custom(<closure>)"),
            MigrationStrategy::NotSupported => write!(f, "NotSupported"),
        }
    }
}

impl Clone for MigrationStrategy {
    fn clone(&self) -> Self {
        match self {
            MigrationStrategy::Direct => MigrationStrategy::Direct,
            MigrationStrategy::Transform(f) => MigrationStrategy::Transform(*f),
            MigrationStrategy::Custom(_) => {
                // Note: Custom closures cannot be cloned, so we return NotSupported
                // This is a design limitation - if cloning is needed for Custom variants,
                // consider using a different approach like an enum of migration types
                MigrationStrategy::NotSupported
            }
            MigrationStrategy::NotSupported => MigrationStrategy::NotSupported,
        }
    }
}

/// Protocol version manager
pub struct VersionManager {
    /// Version registry
    registry: VersionRegistry,
    /// Current protocol version
    current_version: ProtocolVersion,
    /// Version negotiation preferences
    preferences: VersionPreferences,
}

/// Version negotiation preferences
#[derive(Debug, Clone)]
pub struct VersionPreferences {
    /// Preferred version for new connections
    pub preferred_version: ProtocolVersion,
    /// Minimum acceptable version
    pub min_version: ProtocolVersion,
    /// Maximum acceptable version
    pub max_version: ProtocolVersion,
    /// Whether to allow downgrade for compatibility
    pub allow_downgrade: bool,
    /// Features that must be supported
    pub required_features: Vec<String>,
    /// Features that are preferred but not required
    pub preferred_features: Vec<String>,
}

impl VersionRegistry {
    /// Create a new version registry with default versions
    pub fn new() -> Self {
        let mut registry = Self {
            supported_versions: Vec::new(),
            feature_matrix: HashMap::new(),
            message_compatibility: HashMap::new(),
            migration_paths: HashMap::new(),
        };

        // Register default versions
        registry.register_default_versions();
        registry
    }

    /// Register default protocol versions
    fn register_default_versions(&mut self) {
        // Version 1.0.0 - Initial release
        let v1_0_0 = VersionInfo {
            version: ProtocolVersion {
                major: 1,
                minor: 0,
                patch: 0,
                features: vec![],
            },
            name: "Genesis".to_string(),
            release_date: "2024-01-01".to_string(),
            features: vec![
                "basic-messaging".to_string(),
                "quantum-resistant-crypto".to_string(),
                "dag-consensus".to_string(),
                "anonymous-routing".to_string(),
            ],
            deprecated_features: vec![],
            security_requirements: SecurityRequirements {
                min_key_sizes: [("ml-dsa".to_string(), 2048), ("ml-kem".to_string(), 768)]
                    .into_iter()
                    .collect(),
                required_algorithms: vec![
                    "ML-DSA".to_string(),
                    "ML-KEM-768".to_string(),
                    "BLAKE3".to_string(),
                ],
                forbidden_algorithms: vec![
                    "RSA".to_string(),
                    "ECDSA".to_string(),
                    "DH".to_string(),
                ],
                quantum_resistant: true,
            },
            compatibility: CompatibilityInfo {
                compatible_with: vec![],
                min_supported_version: ProtocolVersion {
                    major: 1,
                    minor: 0,
                    patch: 0,
                    features: vec![],
                },
                breaking_changes: vec![],
                migration_notes: vec!["Initial version".to_string()],
            },
        };

        // Version 1.1.0 - Dark addressing support
        let v1_1_0 = VersionInfo {
            version: ProtocolVersion {
                major: 1,
                minor: 1,
                patch: 0,
                features: vec![],
            },
            name: "Shadow".to_string(),
            release_date: "2024-06-01".to_string(),
            features: vec![
                "basic-messaging".to_string(),
                "quantum-resistant-crypto".to_string(),
                "dag-consensus".to_string(),
                "anonymous-routing".to_string(),
                "dark-addressing".to_string(),
                "enhanced-privacy".to_string(),
            ],
            deprecated_features: vec![],
            security_requirements: SecurityRequirements {
                min_key_sizes: [("ml-dsa".to_string(), 2048), ("ml-kem".to_string(), 768)]
                    .into_iter()
                    .collect(),
                required_algorithms: vec![
                    "ML-DSA".to_string(),
                    "ML-KEM-768".to_string(),
                    "BLAKE3".to_string(),
                    "HQC".to_string(),
                ],
                forbidden_algorithms: vec![
                    "RSA".to_string(),
                    "ECDSA".to_string(),
                    "DH".to_string(),
                ],
                quantum_resistant: true,
            },
            compatibility: CompatibilityInfo {
                compatible_with: vec![v1_0_0.version.clone()],
                min_supported_version: v1_0_0.version.clone(),
                breaking_changes: vec![],
                migration_notes: vec![
                    "Backward compatible with 1.0.0".to_string(),
                    "New dark addressing features are optional".to_string(),
                ],
            },
        };

        self.register_version(v1_0_0);
        self.register_version(v1_1_0);

        // Set up migration paths
        self.setup_migration_paths();
    }

    /// Register a new protocol version
    pub fn register_version(&mut self, version_info: VersionInfo) {
        let version = version_info.version.clone();

        // Add to supported versions
        self.supported_versions.push(version_info.clone());

        // Update feature matrix
        let features: HashSet<String> = version_info.features.into_iter().collect();
        self.feature_matrix.insert(version.clone(), features);

        // Update message compatibility (all message types supported by default)
        let message_types: HashSet<MessageType> = [
            MessageType::Handshake(crate::message::HandshakeType::Init),
            MessageType::Handshake(crate::message::HandshakeType::Response),
            MessageType::Handshake(crate::message::HandshakeType::Complete),
            MessageType::Handshake(crate::message::HandshakeType::VersionNegotiation),
            MessageType::Control(crate::message::ControlMessageType::Ping),
            MessageType::Control(crate::message::ControlMessageType::Pong),
            MessageType::Consensus(crate::message::ConsensusMessageType::VertexProposal),
            MessageType::Consensus(crate::message::ConsensusMessageType::Vote),
        ]
        .into_iter()
        .collect();
        self.message_compatibility.insert(version, message_types);
    }

    /// Set up migration paths between versions
    fn setup_migration_paths(&mut self) {
        let v1_0_0 = ProtocolVersion {
            major: 1,
            minor: 0,
            patch: 0,
            features: vec![],
        };
        let v1_1_0 = ProtocolVersion {
            major: 1,
            minor: 1,
            patch: 0,
            features: vec![],
        };

        // Direct migration from 1.0.0 to 1.1.0 (backward compatible)
        self.migration_paths
            .insert((v1_0_0.clone(), v1_1_0.clone()), MigrationStrategy::Direct);

        // Direct migration from 1.1.0 to 1.0.0 (downgrade, remove new features)
        self.migration_paths.insert(
            (v1_1_0, v1_0_0),
            MigrationStrategy::Transform(downgrade_1_1_to_1_0),
        );
    }

    /// Check if a version is supported
    pub fn is_supported(&self, version: &ProtocolVersion) -> bool {
        self.supported_versions
            .iter()
            .any(|v| &v.version == version)
    }

    /// Get version information
    pub fn get_version_info(&self, version: &ProtocolVersion) -> Option<&VersionInfo> {
        self.supported_versions
            .iter()
            .find(|v| &v.version == version)
    }

    /// Get all supported versions
    pub fn get_supported_versions(&self) -> &[VersionInfo] {
        &self.supported_versions
    }

    /// Check if a feature is supported in a version
    pub fn is_feature_supported(&self, version: &ProtocolVersion, feature: &str) -> bool {
        self.feature_matrix
            .get(version)
            .map(|features| features.contains(feature))
            .unwrap_or(false)
    }

    /// Check if two versions are compatible
    pub fn are_compatible(&self, v1: &ProtocolVersion, v2: &ProtocolVersion) -> bool {
        // Same version is always compatible
        if v1 == v2 {
            return true;
        }

        // Check compatibility information
        if let Some(v1_info) = self.get_version_info(v1) {
            if v1_info.compatibility.compatible_with.contains(v2) {
                return true;
            }
        }

        if let Some(v2_info) = self.get_version_info(v2) {
            if v2_info.compatibility.compatible_with.contains(v1) {
                return true;
            }
        }

        // Check if versions follow semantic versioning compatibility
        v1.is_compatible(v2)
    }

    /// Get migration strategy between versions
    pub fn get_migration_strategy(
        &self,
        from: &ProtocolVersion,
        to: &ProtocolVersion,
    ) -> Option<&MigrationStrategy> {
        self.migration_paths.get(&(from.clone(), to.clone()))
    }

    /// Find best compatible version
    pub fn find_best_compatible_version(
        &self,
        available_versions: &[ProtocolVersion],
        preferences: &VersionPreferences,
    ) -> Option<ProtocolVersion> {
        // Filter compatible versions
        let mut compatible: Vec<&ProtocolVersion> = available_versions
            .iter()
            .filter(|v| {
                // Check if version is in acceptable range
                v.major >= preferences.min_version.major &&
                v.major <= preferences.max_version.major &&
                // Check if version is supported
                self.is_supported(v) &&
                // Check required features
                preferences.required_features.iter().all(|feature| {
                    self.is_feature_supported(v, feature)
                })
            })
            .collect();

        if compatible.is_empty() {
            return None;
        }

        // Sort by preference: preferred version first, then highest version
        compatible.sort_by(|a, b| {
            if **a == preferences.preferred_version {
                std::cmp::Ordering::Less
            } else if **b == preferences.preferred_version {
                std::cmp::Ordering::Greater
            } else {
                // Compare by version (highest first)
                b.major
                    .cmp(&a.major)
                    .then(b.minor.cmp(&a.minor))
                    .then(b.patch.cmp(&a.patch))
            }
        });

        Some(compatible[0].clone())
    }
}

impl VersionManager {
    /// Create a new version manager
    pub fn new(current_version: ProtocolVersion) -> Self {
        let preferences = VersionPreferences {
            preferred_version: current_version.clone(),
            min_version: ProtocolVersion {
                major: 1,
                minor: 0,
                patch: 0,
                features: vec![],
            },
            max_version: ProtocolVersion {
                major: 1,
                minor: 1,
                patch: 0,
                features: vec![],
            },
            allow_downgrade: true,
            required_features: vec![
                "quantum-resistant-crypto".to_string(),
                "dag-consensus".to_string(),
            ],
            preferred_features: vec![
                "anonymous-routing".to_string(),
                "dark-addressing".to_string(),
            ],
        };

        Self {
            registry: VersionRegistry::new(),
            current_version,
            preferences,
        }
    }

    /// Negotiate protocol version with peer
    pub fn negotiate_version(
        &self,
        peer_versions: &[ProtocolVersion],
        peer_preferred: &ProtocolVersion,
    ) -> Result<ProtocolVersion, VersionError> {
        // Create combined list of versions to consider
        let mut available_versions = peer_versions.to_vec();
        if !available_versions.contains(peer_preferred) {
            available_versions.push(peer_preferred.clone());
        }

        // Find best compatible version
        if let Some(best_version) = self
            .registry
            .find_best_compatible_version(&available_versions, &self.preferences)
        {
            Ok(best_version)
        } else {
            Err(VersionError::IncompatibleVersions {
                local: self.current_version.clone(),
                remote: peer_preferred.clone(),
            })
        }
    }

    /// Migrate message between protocol versions
    pub fn migrate_message(
        &self,
        message: &Message,
        from_version: &ProtocolVersion,
        to_version: &ProtocolVersion,
    ) -> Result<Message, VersionError> {
        if from_version == to_version {
            return Ok(message.clone());
        }

        if let Some(strategy) = self
            .registry
            .get_migration_strategy(from_version, to_version)
        {
            match strategy {
                MigrationStrategy::Direct => Ok(message.clone()),
                MigrationStrategy::Transform(transform_fn) => transform_fn(message),
                MigrationStrategy::Custom(custom_fn) => custom_fn(message),
                MigrationStrategy::NotSupported => Err(VersionError::MigrationFailed {
                    from: from_version.clone(),
                    to: to_version.clone(),
                    reason: "Migration not supported".to_string(),
                }),
            }
        } else {
            Err(VersionError::MigrationFailed {
                from: from_version.clone(),
                to: to_version.clone(),
                reason: "No migration path found".to_string(),
            })
        }
    }

    /// Check if a feature is available in current version
    pub fn is_feature_available(&self, feature: &str) -> bool {
        self.registry
            .is_feature_supported(&self.current_version, feature)
    }

    /// Get current version
    pub fn current_version(&self) -> &ProtocolVersion {
        &self.current_version
    }

    /// Get version registry
    pub fn registry(&self) -> &VersionRegistry {
        &self.registry
    }

    /// Update preferences
    pub fn set_preferences(&mut self, preferences: VersionPreferences) {
        self.preferences = preferences;
    }

    /// Get preferences
    pub fn preferences(&self) -> &VersionPreferences {
        &self.preferences
    }
}

impl Default for VersionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.features.is_empty() {
            write!(f, "+{}", self.features.join(","))?;
        }
        Ok(())
    }
}

/// Migration function from version 1.1.0 to 1.0.0 (downgrade)
fn downgrade_1_1_to_1_0(message: &Message) -> Result<Message, VersionError> {
    let mut migrated_message = message.clone();

    // Update version
    migrated_message.version = ProtocolVersion {
        major: 1,
        minor: 0,
        patch: 0,
        features: vec![],
    };

    // Remove headers that are specific to 1.1.0
    migrated_message.headers.remove("dark-address");
    migrated_message.headers.remove("shadow-route");

    // For certain message types, we might need to modify the payload
    match &message.msg_type {
        MessageType::Anonymous(_) => {
            // Convert anonymous messages to regular routing messages for 1.0.0
            migrated_message.msg_type =
                MessageType::Routing(crate::message::RoutingMessageType::Direct);
        }
        _ => {
            // Most message types are compatible
        }
    }

    Ok(migrated_message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compatibility() {
        let registry = VersionRegistry::new();

        let v1_0_0 = ProtocolVersion {
            major: 1,
            minor: 0,
            patch: 0,
            features: vec![],
        };
        let v1_1_0 = ProtocolVersion {
            major: 1,
            minor: 1,
            patch: 0,
            features: vec![],
        };

        assert!(registry.are_compatible(&v1_0_0, &v1_1_0));
        assert!(registry.are_compatible(&v1_1_0, &v1_0_0));
    }

    #[test]
    fn test_feature_support() {
        let registry = VersionRegistry::new();

        let v1_0_0 = ProtocolVersion {
            major: 1,
            minor: 0,
            patch: 0,
            features: vec![],
        };
        let v1_1_0 = ProtocolVersion {
            major: 1,
            minor: 1,
            patch: 0,
            features: vec![],
        };

        assert!(registry.is_feature_supported(&v1_0_0, "basic-messaging"));
        assert!(registry.is_feature_supported(&v1_1_0, "dark-addressing"));
        assert!(!registry.is_feature_supported(&v1_0_0, "dark-addressing"));
    }

    #[test]
    fn test_version_negotiation() {
        let manager = VersionManager::new(ProtocolVersion {
            major: 1,
            minor: 1,
            patch: 0,
            features: vec![],
        });

        let peer_versions = vec![
            ProtocolVersion {
                major: 1,
                minor: 0,
                patch: 0,
                features: vec![],
            },
            ProtocolVersion {
                major: 1,
                minor: 1,
                patch: 0,
                features: vec![],
            },
        ];

        let peer_preferred = ProtocolVersion {
            major: 1,
            minor: 1,
            patch: 0,
            features: vec![],
        };

        let negotiated = manager
            .negotiate_version(&peer_versions, &peer_preferred)
            .unwrap();
        assert_eq!(negotiated, peer_preferred);
    }
}
