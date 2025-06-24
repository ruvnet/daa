//! Backward compatibility layer for QuDAG protocol.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::message::{Message, MessageType, ProtocolVersion};
use crate::versioning::{VersionError, VersionManager};

/// Compatibility layer errors
#[derive(Debug, Error)]
pub enum CompatibilityError {
    /// Version not supported
    #[error("Version not supported: {version:?}")]
    VersionNotSupported { version: ProtocolVersion },

    /// Feature not available in target version
    #[error("Feature '{feature}' not available in version {version:?}")]
    FeatureNotAvailable {
        feature: String,
        version: ProtocolVersion,
    },

    /// Message transformation failed
    #[error("Message transformation failed: {reason}")]
    TransformationFailed { reason: String },

    /// Incompatible message format
    #[error("Incompatible message format for version {version:?}")]
    IncompatibleFormat { version: ProtocolVersion },

    /// Version error
    #[error("Version error: {0}")]
    Version(#[from] VersionError),
}

/// Compatibility adapter for handling different protocol versions
pub struct CompatibilityAdapter {
    /// Version manager
    version_manager: VersionManager,
    /// Message transformers for different version pairs
    transformers: HashMap<(ProtocolVersion, ProtocolVersion), Box<dyn MessageTransformer>>,
    /// Feature compatibility matrix
    feature_compatibility: HashMap<ProtocolVersion, Vec<String>>,
}

/// Message transformer trait
pub trait MessageTransformer: Send + Sync {
    /// Transform message from one version to another
    fn transform(&self, message: &Message) -> Result<Message, CompatibilityError>;

    /// Check if transformation is possible
    fn can_transform(&self, message: &Message) -> bool;

    /// Get transformation description
    fn description(&self) -> &str;
}

/// Default message transformer (direct copy with version update)
pub struct DirectTransformer {
    from_version: ProtocolVersion,
    to_version: ProtocolVersion,
}

/// Downgrade transformer (removes unsupported features)
pub struct DowngradeTransformer {
    from_version: ProtocolVersion,
    to_version: ProtocolVersion,
    removed_features: Vec<String>,
}

/// Upgrade transformer (adds default values for new features)
pub struct UpgradeTransformer {
    from_version: ProtocolVersion,
    to_version: ProtocolVersion,
    added_features: Vec<String>,
}

/// Legacy message format for version 1.0.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyMessage {
    /// Message type (simplified)
    pub msg_type: LegacyMessageType,
    /// Message payload
    pub payload: Vec<u8>,
    /// Message timestamp
    pub timestamp: u64,
    /// Message signature
    pub signature: Vec<u8>,
}

/// Legacy message types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegacyMessageType {
    Handshake,
    Data,
    Control,
    Sync,
}

impl CompatibilityAdapter {
    /// Create a new compatibility adapter
    pub fn new(version_manager: VersionManager) -> Self {
        let mut adapter = Self {
            version_manager,
            transformers: HashMap::new(),
            feature_compatibility: HashMap::new(),
        };

        adapter.setup_default_transformers();
        adapter.setup_feature_compatibility();
        adapter
    }

    /// Set up default message transformers
    fn setup_default_transformers(&mut self) {
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

        // Direct transformer for compatible versions
        self.transformers.insert(
            (v1_0_0.clone(), v1_1_0.clone()),
            Box::new(UpgradeTransformer {
                from_version: v1_0_0.clone(),
                to_version: v1_1_0.clone(),
                added_features: vec!["dark-addressing".to_string()],
            }),
        );

        // Downgrade transformer
        self.transformers.insert(
            (v1_1_0.clone(), v1_0_0.clone()),
            Box::new(DowngradeTransformer {
                from_version: v1_1_0,
                to_version: v1_0_0,
                removed_features: vec!["dark-addressing".to_string()],
            }),
        );
    }

    /// Set up feature compatibility matrix
    fn setup_feature_compatibility(&mut self) {
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

        self.feature_compatibility.insert(
            v1_0_0,
            vec![
                "basic-messaging".to_string(),
                "quantum-resistant-crypto".to_string(),
                "dag-consensus".to_string(),
                "anonymous-routing".to_string(),
            ],
        );

        self.feature_compatibility.insert(
            v1_1_0,
            vec![
                "basic-messaging".to_string(),
                "quantum-resistant-crypto".to_string(),
                "dag-consensus".to_string(),
                "anonymous-routing".to_string(),
                "dark-addressing".to_string(),
                "enhanced-privacy".to_string(),
            ],
        );
    }

    /// Transform message to target version
    pub fn transform_message(
        &self,
        message: &Message,
        target_version: &ProtocolVersion,
    ) -> Result<Message, CompatibilityError> {
        if &message.version == target_version {
            return Ok(message.clone());
        }

        // Check if transformation is possible
        if !self
            .version_manager
            .registry()
            .are_compatible(&message.version, target_version)
        {
            return Err(CompatibilityError::VersionNotSupported {
                version: target_version.clone(),
            });
        }

        // Look for direct transformer
        if let Some(transformer) = self
            .transformers
            .get(&(message.version.clone(), target_version.clone()))
        {
            return transformer.transform(message);
        }

        // Use version manager for migration
        self.version_manager
            .migrate_message(message, &message.version, target_version)
            .map_err(CompatibilityError::Version)
    }

    /// Check if feature is available in version
    pub fn is_feature_available(&self, version: &ProtocolVersion, feature: &str) -> bool {
        self.feature_compatibility
            .get(version)
            .map(|features| features.contains(&feature.to_string()))
            .unwrap_or(false)
    }

    /// Get supported features for version
    pub fn get_supported_features(&self, version: &ProtocolVersion) -> Vec<String> {
        self.feature_compatibility
            .get(version)
            .cloned()
            .unwrap_or_default()
    }

    /// Convert modern message to legacy format
    pub fn to_legacy_format(&self, message: &Message) -> Result<LegacyMessage, CompatibilityError> {
        let legacy_type = match &message.msg_type {
            MessageType::Handshake(_) => LegacyMessageType::Handshake,
            MessageType::Control(_) => LegacyMessageType::Control,
            MessageType::Sync(_) => LegacyMessageType::Sync,
            _ => LegacyMessageType::Data,
        };

        Ok(LegacyMessage {
            msg_type: legacy_type,
            payload: message.payload.clone(),
            timestamp: message.timestamp,
            signature: message.signature.clone().unwrap_or_default(),
        })
    }

    /// Convert legacy message to modern format
    pub fn from_legacy_format(
        &self,
        legacy: &LegacyMessage,
    ) -> Result<Message, CompatibilityError> {
        use crate::message::{ControlMessageType, HandshakeType, SyncMessageType};

        let msg_type = match legacy.msg_type {
            LegacyMessageType::Handshake => MessageType::Handshake(HandshakeType::Init),
            LegacyMessageType::Control => MessageType::Control(ControlMessageType::Ping),
            LegacyMessageType::Sync => MessageType::Sync(SyncMessageType::StateRequest),
            LegacyMessageType::Data => {
                MessageType::Routing(crate::message::RoutingMessageType::Direct)
            }
        };

        let mut message = Message::new(msg_type, legacy.payload.clone());
        message.timestamp = legacy.timestamp;
        if !legacy.signature.is_empty() {
            message.signature = Some(legacy.signature.clone());
        }

        // Set version to 1.0.0 for legacy messages
        message.version = ProtocolVersion {
            major: 1,
            minor: 0,
            patch: 0,
            features: vec![],
        };

        Ok(message)
    }

    /// Add custom transformer
    pub fn add_transformer(
        &mut self,
        from_version: ProtocolVersion,
        to_version: ProtocolVersion,
        transformer: Box<dyn MessageTransformer>,
    ) {
        self.transformers
            .insert((from_version, to_version), transformer);
    }

    /// Check compatibility between two versions
    pub fn check_compatibility(
        &self,
        version1: &ProtocolVersion,
        version2: &ProtocolVersion,
    ) -> Result<Vec<String>, CompatibilityError> {
        let mut compatibility_notes = Vec::new();

        if version1 == version2 {
            compatibility_notes.push("Versions are identical".to_string());
            return Ok(compatibility_notes);
        }

        if !self
            .version_manager
            .registry()
            .are_compatible(version1, version2)
        {
            return Err(CompatibilityError::VersionNotSupported {
                version: version2.clone(),
            });
        }

        // Check feature differences
        let features1 = self.get_supported_features(version1);
        let features2 = self.get_supported_features(version2);

        let added_features: Vec<_> = features2
            .iter()
            .filter(|f| !features1.contains(f))
            .cloned()
            .collect();

        let removed_features: Vec<_> = features1
            .iter()
            .filter(|f| !features2.contains(f))
            .cloned()
            .collect();

        if !added_features.is_empty() {
            compatibility_notes.push(format!("Added features: {}", added_features.join(", ")));
        }

        if !removed_features.is_empty() {
            compatibility_notes.push(format!("Removed features: {}", removed_features.join(", ")));
        }

        if added_features.is_empty() && removed_features.is_empty() {
            compatibility_notes.push("No feature differences".to_string());
        }

        Ok(compatibility_notes)
    }
}

impl MessageTransformer for DirectTransformer {
    fn transform(&self, message: &Message) -> Result<Message, CompatibilityError> {
        let mut transformed = message.clone();
        transformed.version = self.to_version.clone();
        Ok(transformed)
    }

    fn can_transform(&self, message: &Message) -> bool {
        message.version == self.from_version
    }

    fn description(&self) -> &str {
        "Direct transformation with version update"
    }
}

impl MessageTransformer for DowngradeTransformer {
    fn transform(&self, message: &Message) -> Result<Message, CompatibilityError> {
        let mut transformed = message.clone();
        transformed.version = self.to_version.clone();

        // Remove headers for unsupported features
        for feature in &self.removed_features {
            if feature.as_str() == "dark-addressing" {
                transformed.headers.remove("dark-address");
                transformed.headers.remove("shadow-route");
            }
        }

        // Transform message types that are not supported in target version
        if let MessageType::Anonymous(_) = &message.msg_type {
            // Convert to direct routing for older versions
            transformed.msg_type = MessageType::Routing(crate::message::RoutingMessageType::Direct);
        }

        Ok(transformed)
    }

    fn can_transform(&self, message: &Message) -> bool {
        message.version == self.from_version
    }

    fn description(&self) -> &str {
        "Downgrade transformation removing unsupported features"
    }
}

impl MessageTransformer for UpgradeTransformer {
    fn transform(&self, message: &Message) -> Result<Message, CompatibilityError> {
        let mut transformed = message.clone();
        transformed.version = self.to_version.clone();

        // Add default values for new features
        for feature in &self.added_features {
            if feature.as_str() == "dark-addressing" {
                // Add default dark addressing headers if not present
                if !transformed.headers.contains_key("addressing-mode") {
                    transformed
                        .headers
                        .insert("addressing-mode".to_string(), "standard".to_string());
                }
            }
        }

        Ok(transformed)
    }

    fn can_transform(&self, message: &Message) -> bool {
        message.version == self.from_version
    }

    fn description(&self) -> &str {
        "Upgrade transformation adding default values for new features"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessageFactory;
    use crate::versioning::VersionManager;

    #[test]
    fn test_version_compatibility() {
        let version_manager = VersionManager::new(ProtocolVersion {
            major: 1,
            minor: 1,
            patch: 0,
            features: vec![],
        });
        let adapter = CompatibilityAdapter::new(version_manager);

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

        let notes = adapter.check_compatibility(&v1_0_0, &v1_1_0).unwrap();
        assert!(!notes.is_empty());
    }

    #[test]
    fn test_message_transformation() {
        let version_manager = VersionManager::new(ProtocolVersion {
            major: 1,
            minor: 1,
            patch: 0,
            features: vec![],
        });
        let adapter = CompatibilityAdapter::new(version_manager);

        let mut message = MessageFactory::create_ping().unwrap();
        message.version = ProtocolVersion {
            major: 1,
            minor: 1,
            patch: 0,
            features: vec![],
        };

        let target_version = ProtocolVersion {
            major: 1,
            minor: 0,
            patch: 0,
            features: vec![],
        };
        let transformed = adapter
            .transform_message(&message, &target_version)
            .unwrap();

        assert_eq!(transformed.version, target_version);
    }

    #[test]
    fn test_legacy_conversion() {
        let version_manager = VersionManager::new(ProtocolVersion {
            major: 1,
            minor: 0,
            patch: 0,
            features: vec![],
        });
        let adapter = CompatibilityAdapter::new(version_manager);

        let message = MessageFactory::create_ping().unwrap();
        let legacy = adapter.to_legacy_format(&message).unwrap();
        let restored = adapter.from_legacy_format(&legacy).unwrap();

        assert_eq!(message.payload, restored.payload);
        assert_eq!(message.timestamp, restored.timestamp);
    }
}
