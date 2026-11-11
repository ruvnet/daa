//! P2P Communication Layer for Decentralized AI Training
//!
//! This module implements a robust peer-to-peer network for distributed
//! gradient sharing and model synchronization.

pub mod behavior;
pub mod compression;
pub mod discovery;
pub mod gradient;
pub mod nat;
pub mod routing;
pub mod transport;

use anyhow::Result;
use libp2p::{
    gossipsub::{Config as GossipsubConfig, Event as GossipsubEvent},
    identify::Event as IdentifyEvent,
    kad::{Config as KademliaConfig, Event as KademliaEvent},
    ping::Event as PingEvent,
    Multiaddr, PeerId, Swarm, SwarmBuilder,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

pub use behavior::{ComposedEvent, NetworkBehavior};
pub use gradient::{AllReduce, GradientMessage};
pub use transport::{create_transport, TransportConfig};

/// Configuration for the P2P swarm
#[derive(Debug, Clone)]
pub struct SwarmConfig {
    /// Local peer ID
    pub local_peer_id: PeerId,
    /// Listen addresses
    pub listen_addresses: Vec<Multiaddr>,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<(PeerId, Multiaddr)>,
    /// Enable mDNS for local discovery
    pub enable_mdns: bool,
    /// Enable NAT traversal
    pub enable_nat_traversal: bool,
    /// Enable relay
    pub enable_relay: bool,
    /// Gossipsub configuration
    pub gossipsub_config: GossipsubConfig,
    /// Kademlia configuration
    pub kademlia_config: KademliaConfig,
    /// Gradient compression level (0-9)
    pub compression_level: u32,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            local_peer_id: PeerId::random(),
            listen_addresses: vec![
                "/ip4/0.0.0.0/tcp/0".parse().unwrap(),
                "/ip4/0.0.0.0/tcp/0/ws".parse().unwrap(),
            ],
            bootstrap_nodes: Vec::new(),
            enable_mdns: true,
            enable_nat_traversal: true,
            enable_relay: true,
            gossipsub_config: GossipsubConfig::default(),
            kademlia_config: KademliaConfig::default(),
            compression_level: 3,
        }
    }
}

/// Main P2P network manager
pub struct P2PNetwork {
    swarm: Swarm<NetworkBehavior>,
    gradient_manager: Arc<RwLock<gradient::GradientManager>>,
    config: SwarmConfig,
}

impl P2PNetwork {
    /// Create a new P2P network
    pub async fn new(config: SwarmConfig) -> Result<Self> {
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!("Local peer ID: {}", local_peer_id);

        // Create transport
        let transport_config = TransportConfig {
            enable_tcp: true,
            enable_websocket: true,
            enable_webrtc: cfg!(feature = "browser"),
            enable_relay: config.enable_relay,
        };
        let transport = create_transport(&local_key, transport_config)?;

        // Create network behavior
        let behavior = NetworkBehavior::new(local_key.clone(), &config)?;

        // Build swarm
        let mut swarm = SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_other_transport(|_| transport)?
            .with_behaviour(|_| behavior)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // Listen on configured addresses
        for addr in &config.listen_addresses {
            swarm.listen_on(addr.clone())?;
            info!("Listening on {}", addr);
        }

        // Create gradient manager
        let gradient_manager = Arc::new(RwLock::new(gradient::GradientManager::new(
            local_peer_id,
            config.compression_level,
        )));

        Ok(Self {
            swarm,
            gradient_manager,
            config,
        })
    }

    /// Connect to bootstrap nodes
    pub async fn bootstrap(&mut self) -> Result<()> {
        for (peer_id, addr) in &self.config.bootstrap_nodes {
            self.swarm.dial(addr.clone())?;
            self.swarm
                .behaviour_mut()
                .kademlia
                .add_address(peer_id, addr.clone());
            info!("Connecting to bootstrap node {} at {}", peer_id, addr);
        }

        // Bootstrap Kademlia
        self.swarm.behaviour_mut().kademlia.bootstrap()?;

        Ok(())
    }

    /// Run the network event loop
    pub async fn run(&mut self) -> Result<()> {
        loop {
            self.process_next_event().await?;
        }
    }

    /// Wait for the next swarm event and handle it.
    ///
    /// Waiting for the event is cancellation-safe; this lets callers drive the
    /// network alongside other work instead of handing it to [`Self::run`].
    pub async fn process_next_event(&mut self) -> Result<()> {
        use futures::StreamExt;

        let event = self.swarm.select_next_some().await;
        match event {
            libp2p::swarm::SwarmEvent::Behaviour(event) => {
                self.handle_behavior_event(event).await?;
            }
            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {}", address);
            }
            libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connected to {}", peer_id);
            }
            libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("Disconnected from {}", peer_id);
            }
            _ => {}
        }
        Ok(())
    }

    /// This node's peer ID.
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// Addresses the swarm is currently listening on.
    pub fn listen_addresses(&self) -> Vec<Multiaddr> {
        self.swarm.listeners().cloned().collect()
    }

    /// Peers with at least one established connection.
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.swarm.connected_peers().copied().collect()
    }

    /// Dial a peer at a known address and record it in the DHT.
    pub fn connect_to_peer(&mut self, peer_id: PeerId, addr: Multiaddr) -> Result<()> {
        self.swarm
            .behaviour_mut()
            .kademlia
            .add_address(&peer_id, addr.clone());
        self.swarm.dial(addr)?;
        Ok(())
    }

    /// Gradients received from other peers for the current round.
    pub async fn received_gradients(&self) -> Vec<(PeerId, Vec<f32>)> {
        let local = self.local_peer_id();
        let manager = self.gradient_manager.read().await;
        let round = manager.current_round();
        manager
            .gradients_for_round(round)
            .await
            .into_iter()
            .filter(|(peer, _)| *peer != local)
            .collect()
    }

    /// Handle behavior events
    async fn handle_behavior_event(&mut self, event: behavior::NetworkBehaviorEvent) -> Result<()> {
        match event {
            behavior::NetworkBehaviorEvent::Kademlia(event) => {
                self.handle_kademlia_event(event).await?
            }
            behavior::NetworkBehaviorEvent::Gossipsub(event) => {
                self.handle_gossipsub_event(event).await?
            }
            behavior::NetworkBehaviorEvent::Identify(event) => {
                self.handle_identify_event(event).await?
            }
            behavior::NetworkBehaviorEvent::Ping(event) => self.handle_ping_event(event).await?,
        }
        Ok(())
    }

    /// Handle Kademlia events
    async fn handle_kademlia_event(&mut self, event: KademliaEvent) -> Result<()> {
        if let KademliaEvent::RoutingUpdated { peer, .. } = event {
            debug!("Routing updated for peer: {}", peer);
        }
        Ok(())
    }

    /// Handle Gossipsub events
    async fn handle_gossipsub_event(&mut self, event: GossipsubEvent) -> Result<()> {
        match event {
            GossipsubEvent::Message {
                propagation_source,
                message_id,
                message,
            } => {
                debug!(
                    "Received message {} from {}",
                    message_id, propagation_source
                );

                // Handle gradient messages
                // Handle gradient messages. Their content comes from remote peers,
                // so a bad message is dropped rather than stopping the event loop.
                if message.topic == gradient::GRADIENT_TOPIC.hash() {
                    let result = match bincode::deserialize::<GradientMessage>(&message.data) {
                        Ok(gradient_msg) => {
                            self.gradient_manager
                                .read()
                                .await
                                .handle_gradient_message(gradient_msg)
                                .await
                        }
                        Err(e) => Err(e.into()),
                    };
                    if let Err(e) = result {
                        warn!(
                            "Dropping gradient message from {}: {}",
                            propagation_source, e
                        );
                    }
                }
            }
            GossipsubEvent::Subscribed { peer_id, topic } => {
                debug!("Peer {} subscribed to topic {:?}", peer_id, topic);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle Identify events
    async fn handle_identify_event(&mut self, event: IdentifyEvent) -> Result<()> {
        if let IdentifyEvent::Received { peer_id, info, .. } = event {
            debug!("Identified peer {}: {:?}", peer_id, info.protocol_version);

            // Add addresses to Kademlia
            for addr in info.listen_addrs {
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, addr);
            }
        }
        Ok(())
    }

    /// Handle Ping events
    async fn handle_ping_event(&mut self, event: PingEvent) -> Result<()> {
        match event.result {
            Ok(duration) => {
                debug!("Ping to {} took {:?}", event.peer, duration);
            }
            Err(e) => {
                warn!("Ping to {} failed: {}", event.peer, e);
            }
        }
        Ok(())
    }

    /// Broadcast a gradient update
    pub async fn broadcast_gradient(&mut self, gradient: Vec<f32>) -> Result<()> {
        let compressed = self
            .gradient_manager
            .read()
            .await
            .compress_gradient(&gradient)?;
        let message = GradientMessage {
            peer_id: *self.swarm.local_peer_id(),
            round: self.gradient_manager.read().await.current_round(),
            compressed_gradient: compressed,
            timestamp: std::time::SystemTime::now(),
        };

        let data = bincode::serialize(&message)?;
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(gradient::GRADIENT_TOPIC.clone(), data)?;

        // Our own contribution takes part in this round's all-reduce too.
        self.gradient_manager
            .read()
            .await
            .handle_gradient_message(message)
            .await?;

        Ok(())
    }

    /// Get the current aggregated gradient
    pub async fn get_aggregated_gradient(&self) -> Result<Option<Vec<f32>>> {
        self.gradient_manager
            .read()
            .await
            .get_aggregated_gradient()
            .await
    }
}
