//! P2P Communication Layer for Decentralized AI Training
//!
//! This module implements a robust peer-to-peer network for distributed
//! gradient sharing and model synchronization.

pub mod behavior;
pub mod transport;
pub mod gradient;
pub mod compression;
pub mod routing;
pub mod discovery;
pub mod nat;

use std::sync::Arc;
use std::time::Duration;
use libp2p::{
    Swarm, SwarmBuilder, PeerId, Multiaddr,
    kad::{Behaviour as Kademlia, Config as KademliaConfig, Event as KademliaEvent, store::MemoryStore},
    gossipsub::{self, Behaviour as Gossipsub, Event as GossipsubEvent, MessageAuthenticity, ValidationMode, Config as GossipsubConfig},
    identify::{Behaviour as Identify, Config as IdentifyConfig, Event as IdentifyEvent},
    ping::{Behaviour as Ping, Event as PingEvent},
    mdns::{tokio::Behaviour as Mdns, Event as MdnsEvent},
    relay,
    autonat,
    upnp,
};
use tokio::sync::{mpsc, RwLock};
use anyhow::Result;
use tracing::{info, debug, warn, error};

pub use behavior::{NetworkBehavior, ComposedEvent};
pub use transport::{TransportConfig, create_transport};
pub use gradient::{GradientMessage, AllReduce};

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
    event_tx: mpsc::UnboundedSender<ComposedEvent>,
    event_rx: mpsc::UnboundedReceiver<ComposedEvent>,
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
        let behavior = NetworkBehavior::new(
            local_key.clone(),
            &config,
        )?;

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

        // Create event channels
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Create gradient manager
        let gradient_manager = Arc::new(RwLock::new(
            gradient::GradientManager::new(local_peer_id, config.compression_level)
        ));

        Ok(Self {
            swarm,
            event_tx,
            event_rx,
            gradient_manager,
            config,
        })
    }

    /// Connect to bootstrap nodes
    pub async fn bootstrap(&mut self) -> Result<()> {
        for (peer_id, addr) in &self.config.bootstrap_nodes {
            self.swarm.dial(addr.clone())?;
            self.swarm.behaviour_mut().kademlia.add_address(peer_id, addr.clone());
            info!("Connecting to bootstrap node {} at {}", peer_id, addr);
        }
        
        // Bootstrap Kademlia
        self.swarm.behaviour_mut().kademlia.bootstrap()?;
        
        Ok(())
    }

    /// Run the network event loop
    pub async fn run(&mut self) -> Result<()> {
        use futures::StreamExt;
        
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
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
                }
            }
        }
    }

    /// Handle behavior events
    async fn handle_behavior_event(&mut self, event: behavior::NetworkBehaviorEvent) -> Result<()> {
        match event {
            behavior::NetworkBehaviorEvent::Kademlia(event) => self.handle_kademlia_event(event).await?,
            behavior::NetworkBehaviorEvent::Gossipsub(event) => self.handle_gossipsub_event(event).await?,
            behavior::NetworkBehaviorEvent::Identify(event) => self.handle_identify_event(event).await?,
            behavior::NetworkBehaviorEvent::Ping(event) => self.handle_ping_event(event).await?,
        }
        Ok(())
    }

    /// Handle Kademlia events
    async fn handle_kademlia_event(&mut self, event: KademliaEvent) -> Result<()> {
        use libp2p::kad::{QueryResult, GetClosestPeersOk};
        
        match event {
            KademliaEvent::RoutingUpdated { peer, .. } => {
                debug!("Routing updated for peer: {}", peer);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle Gossipsub events
    async fn handle_gossipsub_event(&mut self, event: GossipsubEvent) -> Result<()> {
        match event {
            GossipsubEvent::Message { propagation_source, message_id, message } => {
                debug!("Received message {} from {}", message_id, propagation_source);
                
                // Handle gradient messages
                if message.topic == gradient::GRADIENT_TOPIC.hash() {
                    let gradient_msg: GradientMessage = bincode::deserialize(&message.data)?;
                    self.gradient_manager.write().await.handle_gradient_message(gradient_msg).await?;
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
        match event {
            IdentifyEvent::Received { peer_id, info } => {
                debug!("Identified peer {}: {:?}", peer_id, info.protocol_version);
                
                // Add addresses to Kademlia
                for addr in info.listen_addrs {
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
            }
            _ => {}
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

    /// Handle mDNS events
    async fn handle_mdns_event(&mut self, event: MdnsEvent) -> Result<()> {
        match event {
            MdnsEvent::Discovered(peers) => {
                for (peer_id, addr) in peers {
                    info!("Discovered peer {} at {} via mDNS", peer_id, addr);
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                    self.swarm.dial(addr)?;
                }
            }
            MdnsEvent::Expired(peers) => {
                for (peer_id, _) in peers {
                    debug!("mDNS peer expired: {}", peer_id);
                }
            }
        }
        Ok(())
    }

    /// Broadcast a gradient update
    pub async fn broadcast_gradient(&mut self, gradient: Vec<f32>) -> Result<()> {
        let compressed = self.gradient_manager.read().await.compress_gradient(&gradient)?;
        let message = GradientMessage {
            peer_id: self.swarm.local_peer_id().clone(),
            round: self.gradient_manager.read().await.current_round(),
            compressed_gradient: compressed,
            timestamp: std::time::SystemTime::now(),
        };
        
        let data = bincode::serialize(&message)?;
        self.swarm.behaviour_mut().gossipsub.publish(gradient::GRADIENT_TOPIC.clone(), data)?;
        
        Ok(())
    }

    /// Get the current aggregated gradient
    pub async fn get_aggregated_gradient(&self) -> Result<Option<Vec<f32>>> {
        self.gradient_manager.read().await.get_aggregated_gradient().await
    }
}