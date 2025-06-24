//! Protocol state machine implementation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};
use thiserror::Error;
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::message::{HandshakeType, Message, MessageType, ProtocolVersion};

/// Errors that can occur during state operations.
#[derive(Debug, Error)]
pub enum StateError {
    /// Invalid state transition
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: ProtocolState,
        to: ProtocolState,
    },

    /// State synchronization failed
    #[error("State synchronization failed: {reason}")]
    SyncFailed { reason: String },

    /// Invalid state data
    #[error("Invalid state data: {reason}")]
    InvalidData { reason: String },

    /// State operation timeout
    #[error("State operation timed out after {timeout:?}")]
    Timeout { timeout: Duration },

    /// Session not found
    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: Uuid },

    /// Protocol version mismatch
    #[error("Protocol version mismatch: expected {expected:?}, got {actual:?}")]
    VersionMismatch {
        expected: ProtocolVersion,
        actual: ProtocolVersion,
    },
}

/// Protocol state enumeration with detailed substates
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProtocolState {
    /// Initial state - node starting up
    #[default]
    Initial,

    /// Handshake states
    Handshake(HandshakeState),

    /// Active protocol states
    Active(ActiveState),

    /// Synchronization states
    Synchronizing(SyncState),

    /// Error states
    Error(ErrorState),

    /// Shutting down
    Shutdown,
}

/// Handshake state substates
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandshakeState {
    /// Waiting for handshake initiation
    Waiting,
    /// Handshake in progress - sent init, waiting for response
    InProgress,
    /// Received handshake response, processing
    Processing,
    /// Handshake completed successfully
    Completed,
    /// Handshake failed
    Failed,
}

/// Active protocol substates
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveState {
    /// Normal operation - processing messages
    Normal,
    /// High load - prioritizing critical messages
    HighLoad,
    /// Degraded - some components not functioning optimally
    Degraded,
}

/// Synchronization substates
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncState {
    /// Requesting state from peers
    Requesting,
    /// Receiving state data
    Receiving,
    /// Applying received state
    Applying,
    /// Verifying synchronized state
    Verifying,
}

/// Error state types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorState {
    /// Network connectivity issues
    NetworkError,
    /// Consensus failure
    ConsensusError,
    /// Cryptographic error
    CryptoError,
    /// Resource exhaustion
    ResourceError,
    /// Internal protocol error
    InternalError,
}

/// Session information for tracking peer connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session identifier
    pub id: Uuid,
    /// Peer identifier
    pub peer_id: Vec<u8>,
    /// Protocol version negotiated
    pub protocol_version: ProtocolVersion,
    /// Session state
    pub state: ProtocolState,
    /// Session start time
    pub started_at: SystemTime,
    /// Last activity timestamp
    pub last_activity: SystemTime,
    /// Session capabilities
    pub capabilities: Vec<String>,
    /// Session metrics
    pub metrics: SessionMetrics,
}

/// Session performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Messages sent
    pub messages_sent: u64,
    /// Messages received
    pub messages_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Average response time
    pub avg_response_time: Duration,
    /// Error count
    pub error_count: u64,
}

/// Protocol state manager
#[derive(Debug)]
pub struct ProtocolStateMachine {
    /// Current protocol state
    current_state: ProtocolState,
    /// Previous state for rollback
    previous_state: Option<ProtocolState>,
    /// State transition history
    state_history: Vec<StateTransition>,
    /// Active sessions
    sessions: HashMap<Uuid, SessionInfo>,
    /// State machine start time
    started_at: Instant,
    /// Protocol version
    protocol_version: ProtocolVersion,
    /// State machine configuration
    config: StateMachineConfig,
}

/// State transition record
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// Timestamp of transition
    pub timestamp: Instant,
    /// Previous state
    pub from: ProtocolState,
    /// New state
    pub to: ProtocolState,
    /// Reason for transition
    pub reason: String,
    /// Duration in previous state
    pub duration: Duration,
}

/// State machine configuration
#[derive(Debug, Clone)]
pub struct StateMachineConfig {
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    /// Session timeout duration
    pub session_timeout: Duration,
    /// Handshake timeout duration
    pub handshake_timeout: Duration,
    /// Sync timeout duration
    pub sync_timeout: Duration,
    /// Maximum state history size
    pub max_history_size: usize,
}

impl Default for StateMachineConfig {
    fn default() -> Self {
        Self {
            max_sessions: 1000,
            session_timeout: Duration::from_secs(300), // 5 minutes
            handshake_timeout: Duration::from_secs(30), // 30 seconds
            sync_timeout: Duration::from_secs(60),     // 1 minute
            max_history_size: 1000,
        }
    }
}

impl ProtocolStateMachine {
    /// Create a new protocol state machine
    pub fn new(protocol_version: ProtocolVersion) -> Self {
        Self {
            current_state: ProtocolState::Initial,
            previous_state: None,
            state_history: Vec::new(),
            sessions: HashMap::new(),
            started_at: Instant::now(),
            protocol_version,
            config: StateMachineConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(protocol_version: ProtocolVersion, config: StateMachineConfig) -> Self {
        Self {
            current_state: ProtocolState::Initial,
            previous_state: None,
            state_history: Vec::new(),
            sessions: HashMap::new(),
            started_at: Instant::now(),
            protocol_version,
            config,
        }
    }

    /// Get current state
    pub fn current_state(&self) -> &ProtocolState {
        &self.current_state
    }

    /// Get active sessions count
    pub fn active_sessions(&self) -> usize {
        self.sessions.len()
    }

    /// Get protocol version
    pub fn protocol_version(&self) -> &ProtocolVersion {
        &self.protocol_version
    }

    /// Get state machine uptime
    pub fn uptime(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Transition to a new state
    pub fn transition_to(
        &mut self,
        new_state: ProtocolState,
        reason: String,
    ) -> Result<(), StateError> {
        // Validate transition
        if !self.is_valid_transition(&self.current_state, &new_state) {
            return Err(StateError::InvalidTransition {
                from: self.current_state.clone(),
                to: new_state,
            });
        }

        let now = Instant::now();
        let duration = if let Some(last_transition) = self.state_history.last() {
            now.duration_since(last_transition.timestamp)
        } else {
            now.duration_since(self.started_at)
        };

        // Record state transition
        let transition = StateTransition {
            timestamp: now,
            from: self.current_state.clone(),
            to: new_state.clone(),
            reason: reason.clone(),
            duration,
        };

        debug!(
            "State transition: {:?} -> {:?} ({})",
            self.current_state, new_state, reason
        );

        // Update states
        self.previous_state = Some(self.current_state.clone());
        self.current_state = new_state;

        // Add to history
        self.state_history.push(transition);

        // Limit history size
        if self.state_history.len() > self.config.max_history_size {
            self.state_history.remove(0);
        }

        // Perform state entry actions
        self.on_state_entry(&reason)?;

        Ok(())
    }

    /// Check if a state transition is valid
    fn is_valid_transition(&self, from: &ProtocolState, to: &ProtocolState) -> bool {
        use ActiveState::*;
        use ErrorState::*;
        use HandshakeState::*;
        use ProtocolState::*;
        use SyncState::*;

        match (from, to) {
            // From Initial
            (Initial, Handshake(Waiting)) => true,
            (Initial, Error(_)) => true,
            (Initial, Shutdown) => true,

            // From Handshake states
            (Handshake(Waiting), Handshake(InProgress)) => true,
            (Handshake(InProgress), Handshake(Processing)) => true,
            (Handshake(InProgress), Handshake(Failed)) => true,
            (Handshake(Processing), Handshake(Completed)) => true,
            (Handshake(Processing), Handshake(Failed)) => true,
            (Handshake(Completed), Active(Normal)) => true,
            (Handshake(Failed), Error(NetworkError)) => true,
            (Handshake(_), Shutdown) => true,

            // From Active states
            (Active(Normal), Active(HighLoad)) => true,
            (Active(Normal), Active(Degraded)) => true,
            (Active(Normal), Synchronizing(Requesting)) => true,
            (Active(HighLoad), Active(Normal)) => true,
            (Active(HighLoad), Active(Degraded)) => true,
            (Active(Degraded), Active(Normal)) => true,
            (Active(Degraded), Synchronizing(Requesting)) => true,
            (Active(_), Error(_)) => true,
            (Active(_), Shutdown) => true,

            // From Synchronizing states
            (Synchronizing(Requesting), Synchronizing(Receiving)) => true,
            (Synchronizing(Requesting), Error(NetworkError)) => true,
            (Synchronizing(Receiving), Synchronizing(Applying)) => true,
            (Synchronizing(Receiving), Error(NetworkError)) => true,
            (Synchronizing(Applying), Synchronizing(Verifying)) => true,
            (Synchronizing(Applying), Error(InternalError)) => true,
            (Synchronizing(Verifying), Active(Normal)) => true,
            (Synchronizing(Verifying), Error(InternalError)) => true,
            (Synchronizing(_), Shutdown) => true,

            // From Error states
            (Error(_), Initial) => true, // Recovery
            (Error(_), Shutdown) => true,

            // From Shutdown
            (Shutdown, _) => false, // No transitions from shutdown

            // Same state (for updates)
            (a, b) if a == b => true,

            _ => false,
        }
    }

    /// Perform actions when entering a new state
    fn on_state_entry(&mut self, reason: &str) -> Result<(), StateError> {
        let current_state = self.current_state.clone();
        match &current_state {
            ProtocolState::Initial => {
                debug!("Entered Initial state: {}", reason);
                // Clean up any existing sessions
                self.sessions.clear();
            }

            ProtocolState::Handshake(handshake_state) => {
                debug!("Entered Handshake state {:?}: {}", handshake_state, reason);
                match handshake_state {
                    HandshakeState::InProgress => {
                        // Start handshake timeout
                        // TODO: Implement timeout mechanism
                    }
                    HandshakeState::Failed => {
                        warn!("Handshake failed: {}", reason);
                        // Clean up failed handshake sessions
                        self.cleanup_failed_sessions();
                    }
                    _ => {}
                }
            }

            ProtocolState::Active(active_state) => {
                debug!("Entered Active state {:?}: {}", active_state, reason);
                match active_state {
                    ActiveState::HighLoad => {
                        // Implement load shedding
                        warn!("Entering high load state, implementing load shedding");
                    }
                    ActiveState::Degraded => {
                        warn!("Entering degraded state: {}", reason);
                    }
                    _ => {}
                }
            }

            ProtocolState::Synchronizing(sync_state) => {
                debug!("Entered Synchronizing state {:?}: {}", sync_state, reason);
            }

            ProtocolState::Error(error_state) => {
                error!("Entered Error state {:?}: {}", error_state, reason);
                // Implement error recovery procedures
                self.handle_error_state(error_state, reason)?;
            }

            ProtocolState::Shutdown => {
                debug!("Entered Shutdown state: {}", reason);
                // Begin graceful shutdown
                self.begin_shutdown();
            }
        }

        Ok(())
    }

    /// Handle error state entry
    fn handle_error_state(
        &mut self,
        error_state: &ErrorState,
        reason: &str,
    ) -> Result<(), StateError> {
        match error_state {
            ErrorState::NetworkError => {
                // Close problematic connections
                self.cleanup_failed_sessions();
            }
            ErrorState::ConsensusError => {
                // Request state synchronization
                // TODO: Trigger sync process
            }
            ErrorState::CryptoError => {
                // Critical error - may need to restart
                error!("Critical cryptographic error: {}", reason);
            }
            ErrorState::ResourceError => {
                // Implement resource cleanup
                self.cleanup_resources();
            }
            ErrorState::InternalError => {
                // Log detailed error information
                error!("Internal protocol error: {}", reason);
            }
        }
        Ok(())
    }

    /// Begin graceful shutdown
    fn begin_shutdown(&mut self) {
        debug!("Beginning graceful shutdown");

        // Close all active sessions
        for (session_id, session) in &mut self.sessions {
            debug!("Closing session: {}", session_id);
            session.state = ProtocolState::Shutdown;
        }

        // TODO: Send disconnect messages to peers
        // TODO: Flush pending operations
    }

    /// Clean up failed sessions
    fn cleanup_failed_sessions(&mut self) {
        let failed_sessions: Vec<Uuid> = self
            .sessions
            .iter()
            .filter(|(_, session)| {
                matches!(
                    session.state,
                    ProtocolState::Error(_) | ProtocolState::Handshake(HandshakeState::Failed)
                )
            })
            .map(|(id, _)| *id)
            .collect();

        for session_id in failed_sessions {
            debug!("Cleaning up failed session: {}", session_id);
            self.sessions.remove(&session_id);
        }
    }

    /// Clean up resources
    fn cleanup_resources(&mut self) {
        debug!("Cleaning up resources");

        // Remove old state history
        if self.state_history.len() > self.config.max_history_size / 2 {
            let keep_from = self.state_history.len() - self.config.max_history_size / 2;
            self.state_history.drain(0..keep_from);
        }

        // Remove timed out sessions
        self.cleanup_timed_out_sessions();
    }

    /// Clean up timed out sessions
    fn cleanup_timed_out_sessions(&mut self) {
        let now = SystemTime::now();
        let timeout = self.config.session_timeout;

        let timed_out_sessions: Vec<Uuid> = self
            .sessions
            .iter()
            .filter(|(_, session)| {
                now.duration_since(session.last_activity)
                    .unwrap_or(Duration::ZERO)
                    > timeout
            })
            .map(|(id, _)| *id)
            .collect();

        for session_id in timed_out_sessions {
            debug!("Removing timed out session: {}", session_id);
            self.sessions.remove(&session_id);
        }
    }

    /// Create a new session
    pub fn create_session(
        &mut self,
        peer_id: Vec<u8>,
        protocol_version: ProtocolVersion,
        capabilities: Vec<String>,
    ) -> Result<Uuid, StateError> {
        // Check session limit
        if self.sessions.len() >= self.config.max_sessions {
            return Err(StateError::InvalidData {
                reason: "Maximum number of sessions reached".to_string(),
            });
        }

        // Verify protocol version compatibility
        if !self.protocol_version.is_compatible(&protocol_version) {
            return Err(StateError::VersionMismatch {
                expected: self.protocol_version.clone(),
                actual: protocol_version,
            });
        }

        let session_id = Uuid::new_v4();
        let now = SystemTime::now();

        let session = SessionInfo {
            id: session_id,
            peer_id,
            protocol_version,
            state: ProtocolState::Handshake(HandshakeState::Waiting),
            started_at: now,
            last_activity: now,
            capabilities,
            metrics: SessionMetrics::default(),
        };

        self.sessions.insert(session_id, session);

        debug!("Created new session: {}", session_id);
        Ok(session_id)
    }

    /// Update session state
    pub fn update_session_state(
        &mut self,
        session_id: Uuid,
        new_state: ProtocolState,
    ) -> Result<(), StateError> {
        // First get the current session state for validation
        let current_session_state = self
            .sessions
            .get(&session_id)
            .ok_or(StateError::SessionNotFound { session_id })?
            .state
            .clone();

        // Validate session state transition
        if !self.is_valid_transition(&current_session_state, &new_state) {
            return Err(StateError::InvalidTransition {
                from: current_session_state,
                to: new_state,
            });
        }

        // Now get mutable reference and update
        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or(StateError::SessionNotFound { session_id })?;
        session.state = new_state;
        session.last_activity = SystemTime::now();

        Ok(())
    }

    /// Get session information
    pub fn get_session(&self, session_id: &Uuid) -> Option<&SessionInfo> {
        self.sessions.get(session_id)
    }

    /// Remove a session
    pub fn remove_session(&mut self, session_id: &Uuid) -> Option<SessionInfo> {
        self.sessions.remove(session_id)
    }

    /// Process a protocol message and update state accordingly
    pub fn process_message(
        &mut self,
        message: &Message,
        session_id: Option<Uuid>,
    ) -> Result<(), StateError> {
        // Update last activity for session if provided
        if let Some(session_id) = session_id {
            if let Some(session) = self.sessions.get_mut(&session_id) {
                session.last_activity = SystemTime::now();
                session.metrics.messages_received += 1;
                session.metrics.bytes_received += message.payload.len() as u64;
            }
        }

        // Process message based on type and current state
        match &message.msg_type {
            MessageType::Handshake(handshake_type) => {
                self.process_handshake_message(handshake_type, message, session_id)?;
            }
            MessageType::Control(_) => {
                // Control messages can be processed in most states
                if !matches!(self.current_state, ProtocolState::Shutdown) {
                    // Process control message
                    debug!(
                        "Processing control message in state {:?}",
                        self.current_state
                    );
                }
            }
            _ => {
                // Other messages require active state
                match &self.current_state {
                    ProtocolState::Active(_) => {
                        // Process message in active state
                        debug!("Processing message in active state");
                    }
                    _ => {
                        warn!(
                            "Received message in non-active state: {:?}",
                            self.current_state
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Process handshake messages
    fn process_handshake_message(
        &mut self,
        handshake_type: &HandshakeType,
        _message: &Message,
        session_id: Option<Uuid>,
    ) -> Result<(), StateError> {
        match handshake_type {
            HandshakeType::Init => {
                if matches!(
                    self.current_state,
                    ProtocolState::Initial | ProtocolState::Handshake(_)
                ) {
                    self.transition_to(
                        ProtocolState::Handshake(HandshakeState::InProgress),
                        "Received handshake init".to_string(),
                    )?;
                }
            }
            HandshakeType::Response => {
                if matches!(
                    self.current_state,
                    ProtocolState::Handshake(HandshakeState::InProgress)
                ) {
                    self.transition_to(
                        ProtocolState::Handshake(HandshakeState::Processing),
                        "Received handshake response".to_string(),
                    )?;
                }
            }
            HandshakeType::Complete => {
                if matches!(
                    self.current_state,
                    ProtocolState::Handshake(HandshakeState::Processing)
                ) {
                    self.transition_to(
                        ProtocolState::Handshake(HandshakeState::Completed),
                        "Handshake completed".to_string(),
                    )?;

                    // Transition to active state
                    self.transition_to(
                        ProtocolState::Active(ActiveState::Normal),
                        "Handshake successful, entering active state".to_string(),
                    )?;
                }
            }
            HandshakeType::VersionNegotiation => {
                // Handle version negotiation
                debug!("Processing version negotiation");
            }
        }

        // Update session state if provided
        if let Some(session_id) = session_id {
            if let Some(session) = self.sessions.get_mut(&session_id) {
                session.state = self.current_state.clone();
            }
        }

        Ok(())
    }

    /// Get state transition history
    pub fn get_state_history(&self) -> &[StateTransition] {
        &self.state_history
    }

    /// Get all active sessions
    pub fn get_sessions(&self) -> &HashMap<Uuid, SessionInfo> {
        &self.sessions
    }

    /// Check if the state machine is in a healthy state
    pub fn is_healthy(&self) -> bool {
        !matches!(
            self.current_state,
            ProtocolState::Error(_) | ProtocolState::Shutdown
        )
    }

    /// Get state machine metrics
    pub fn get_metrics(&self) -> StateMachineMetrics {
        let mut total_messages_sent = 0;
        let mut total_messages_received = 0;
        let mut total_bytes_sent = 0;
        let mut total_bytes_received = 0;
        let mut total_errors = 0;

        for session in self.sessions.values() {
            total_messages_sent += session.metrics.messages_sent;
            total_messages_received += session.metrics.messages_received;
            total_bytes_sent += session.metrics.bytes_sent;
            total_bytes_received += session.metrics.bytes_received;
            total_errors += session.metrics.error_count;
        }

        StateMachineMetrics {
            current_state: self.current_state.clone(),
            uptime: self.uptime(),
            active_sessions: self.sessions.len(),
            total_state_transitions: self.state_history.len(),
            total_messages_sent,
            total_messages_received,
            total_bytes_sent,
            total_bytes_received,
            total_errors,
        }
    }
}

/// State machine performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateMachineMetrics {
    /// Current protocol state
    pub current_state: ProtocolState,
    /// State machine uptime
    pub uptime: Duration,
    /// Number of active sessions
    pub active_sessions: usize,
    /// Total state transitions
    pub total_state_transitions: usize,
    /// Total messages sent across all sessions
    pub total_messages_sent: u64,
    /// Total messages received across all sessions
    pub total_messages_received: u64,
    /// Total bytes sent
    pub total_bytes_sent: u64,
    /// Total bytes received
    pub total_bytes_received: u64,
    /// Total errors
    pub total_errors: u64,
}

/// State management trait defining the interface for state operations.
pub trait StateManager {
    /// Initialize protocol state.
    fn init() -> Result<ProtocolStateMachine, StateError>;

    /// Transition to a new state.
    fn transition(&mut self, new_state: ProtocolState) -> Result<(), StateError>;

    /// Get current state.
    fn get_state(&self) -> &ProtocolState;

    /// Validate state transition.
    fn validate_transition(&self, new_state: &ProtocolState) -> bool;
}

impl StateManager for ProtocolStateMachine {
    fn init() -> Result<ProtocolStateMachine, StateError> {
        Ok(ProtocolStateMachine::new(ProtocolVersion::CURRENT))
    }

    fn transition(&mut self, new_state: ProtocolState) -> Result<(), StateError> {
        self.transition_to(new_state, "Manual transition".to_string())
    }

    fn get_state(&self) -> &ProtocolState {
        &self.current_state
    }

    fn validate_transition(&self, new_state: &ProtocolState) -> bool {
        self.is_valid_transition(&self.current_state, new_state)
    }
}
