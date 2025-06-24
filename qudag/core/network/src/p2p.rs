use either::Either;
use libp2p::{
    core::{
        multiaddr::{Multiaddr, Protocol},
        transport::{Boxed, MemoryTransport, Transport as LibP2PTransport},
        upgrade::{self},
    },
    dcutr,
    gossipsub::{
        self, Config as GossipsubConfig, ConfigBuilder as GossipsubConfigBuilder, IdentTopic,
        MessageAuthenticity, ValidationMode,
    },
    identify::{self},
    identity::{self, Keypair},
    kad::{self, store::MemoryStore, QueryResult},
    mdns::{self},
    noise,
    ping::{self},
    relay,
    request_response::{self, ProtocolSupport},
    swarm::{behaviour::toggle::Toggle, NetworkBehaviour, SwarmEvent},
    tcp, websocket, yamux, PeerId as LibP2PPeerId, StreamProtocol,
};
use void;

/// Combined network behaviour event
#[derive(Debug)]
pub enum NetworkBehaviourEvent {
    Kademlia(kad::Event),
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Ping(ping::Event),
    Identify(identify::Event),
    Relay(relay::Event),
    Dcutr(dcutr::Event),
    RequestResponse(request_response::Event<QuDagRequest, QuDagResponse>),
}

// Implement From traits for all event types
impl From<kad::Event> for NetworkBehaviourEvent {
    fn from(event: kad::Event) -> Self {
        NetworkBehaviourEvent::Kademlia(event)
    }
}

impl From<gossipsub::Event> for NetworkBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        NetworkBehaviourEvent::Gossipsub(event)
    }
}

impl From<mdns::Event> for NetworkBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        NetworkBehaviourEvent::Mdns(event)
    }
}

// Handle Toggle<T> event conversion for MDNS
impl From<Either<mdns::Event, void::Void>> for NetworkBehaviourEvent {
    fn from(event: Either<mdns::Event, void::Void>) -> Self {
        match event {
            Either::Left(mdns_event) => NetworkBehaviourEvent::Mdns(mdns_event),
            Either::Right(void) => match void {},
        }
    }
}

impl From<ping::Event> for NetworkBehaviourEvent {
    fn from(event: ping::Event) -> Self {
        NetworkBehaviourEvent::Ping(event)
    }
}

impl From<identify::Event> for NetworkBehaviourEvent {
    fn from(event: identify::Event) -> Self {
        NetworkBehaviourEvent::Identify(event)
    }
}

impl From<relay::Event> for NetworkBehaviourEvent {
    fn from(event: relay::Event) -> Self {
        NetworkBehaviourEvent::Relay(event)
    }
}

impl From<dcutr::Event> for NetworkBehaviourEvent {
    fn from(event: dcutr::Event) -> Self {
        NetworkBehaviourEvent::Dcutr(event)
    }
}

impl From<request_response::Event<QuDagRequest, QuDagResponse>> for NetworkBehaviourEvent {
    fn from(event: request_response::Event<QuDagRequest, QuDagResponse>) -> Self {
        NetworkBehaviourEvent::RequestResponse(event)
    }
}

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use futures::{channel::oneshot, prelude::*};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, warn};

use crate::routing::Router;
// Optimization features disabled for initial release
// use crate::optimized::message_chunking::{MessageChunker, ChunkerConfig, ChunkedMessage};
use crate::types::{MessagePriority, NetworkMessage};

/// Configuration for the P2P network node
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Local listening addresses
    pub listen_addrs: Vec<String>,
    /// Bootstrap peer addresses
    pub bootstrap_peers: Vec<String>,
    /// Connection timeout
    pub timeout: Duration,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Traffic obfuscation key
    pub obfuscation_key: [u8; 32],
    /// Enable MDNS for local peer discovery
    pub enable_mdns: bool,
    /// Enable relay for NAT traversal
    pub enable_relay: bool,
    /// Enable QUIC transport
    pub enable_quic: bool,
    /// Enable WebSocket transport
    pub enable_websocket: bool,
    /// Gossipsub configuration
    pub gossipsub_config: Option<GossipsubConfig>,
    /// Kademlia replication factor
    pub kad_replication_factor: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        let mut key = [0u8; 32];
        thread_rng().fill_bytes(&mut key);

        Self {
            listen_addrs: vec![
                "/ip4/0.0.0.0/tcp/0".to_string(),
                "/ip6/::/tcp/0".to_string(),
            ],
            bootstrap_peers: vec![],
            timeout: Duration::from_secs(20),
            max_connections: 50,
            obfuscation_key: key,
            enable_mdns: true,
            enable_relay: true,
            enable_quic: false,
            enable_websocket: true,
            gossipsub_config: None,
            kad_replication_factor: 20,
        }
    }
}

/// Request-response protocol for custom messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuDagRequest {
    pub request_id: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuDagResponse {
    pub request_id: String,
    pub payload: Vec<u8>,
}

/// Combined network behaviour for the P2P node
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "NetworkBehaviourEvent")]
pub struct NetworkBehaviourImpl {
    /// Kademlia DHT for peer discovery and content routing
    pub kademlia: kad::Behaviour<MemoryStore>,
    /// Gossipsub for pub/sub messaging
    pub gossipsub: gossipsub::Behaviour,
    /// MDNS for local peer discovery
    pub mdns: Toggle<mdns::tokio::Behaviour>,
    /// Ping for keep-alive and latency measurement
    pub ping: ping::Behaviour,
    /// Identify protocol for peer identification
    pub identify: identify::Behaviour,
    /// Relay for NAT traversal
    pub relay: relay::Behaviour,
    /// Direct connection upgrade through relay
    pub dcutr: dcutr::Behaviour,
    /// Request-response protocol for custom messages
    pub request_response: request_response::cbor::Behaviour<QuDagRequest, QuDagResponse>,
}

/// Commands that can be sent to the P2P node
#[derive(Debug)]
pub enum P2PCommand {
    /// Subscribe to a gossipsub topic
    Subscribe {
        topic: String,
        response: oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    },
    /// Unsubscribe from a gossipsub topic
    Unsubscribe {
        topic: String,
        response: oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    },
    /// Publish a message to a topic
    Publish {
        topic: String,
        data: Vec<u8>,
        response: oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    },
    /// Send a request to a peer
    SendRequest {
        peer_id: LibP2PPeerId,
        request: QuDagRequest,
        response: oneshot::Sender<Result<QuDagResponse, Box<dyn Error + Send + Sync>>>,
    },
    /// Dial a peer
    Dial {
        addr: Multiaddr,
        response: oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    },
    /// Get connected peers
    GetConnectedPeers {
        response: oneshot::Sender<Vec<LibP2PPeerId>>,
    },
    /// Get local peer ID
    GetLocalPeerId {
        response: oneshot::Sender<LibP2PPeerId>,
    },
    /// Get listeners
    GetListeners {
        response: oneshot::Sender<Vec<Multiaddr>>,
    },
}

/// Events emitted by the P2P network
#[derive(Debug)]
pub enum P2PEvent {
    /// New peer discovered
    PeerDiscovered(LibP2PPeerId),
    /// Peer connection established
    PeerConnected(LibP2PPeerId),
    /// Peer disconnected
    PeerDisconnected(LibP2PPeerId),
    /// Message received via gossipsub
    MessageReceived {
        peer_id: LibP2PPeerId,
        topic: String,
        data: Vec<u8>,
    },
    /// Request received
    RequestReceived {
        peer_id: LibP2PPeerId,
        request: QuDagRequest,
        channel: oneshot::Sender<QuDagResponse>,
    },
    /// Response received
    ResponseReceived {
        peer_id: LibP2PPeerId,
        response: QuDagResponse,
    },
    /// Routing table updated
    RoutingTableUpdated,
}

/// Main P2P network node implementation
pub struct P2PNode {
    /// Local peer ID
    local_peer_id: LibP2PPeerId,
    /// Swarm instance
    swarm: libp2p::Swarm<NetworkBehaviourImpl>,
    /// Router for message routing  
    router: Router,
    /// Traffic obfuscation cipher
    cipher: ChaCha20Poly1305,
    /// Event channel sender
    event_tx: mpsc::UnboundedSender<P2PEvent>,
    /// Command channel receiver
    command_rx: mpsc::UnboundedReceiver<P2PCommand>,
    /// Connected peers
    connected_peers: HashSet<LibP2PPeerId>,
    /// Pending requests
    pending_requests: HashMap<String, oneshot::Sender<QuDagResponse>>,
    /// Metrics recorder
    #[allow(dead_code)]
    metrics: Option<()>, // TODO: Use proper metrics type
    /// Network configuration
    config: NetworkConfig,
    // Message chunker for large messages (disabled for initial release)
    // message_chunker: MessageChunker,
}

/// Handle for sending commands to the P2P node
#[derive(Clone)]
pub struct P2PHandle {
    /// Command channel sender
    command_tx: mpsc::UnboundedSender<P2PCommand>,
    /// Event channel receiver (cloned for each handle)
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<P2PEvent>>>,
}

impl P2PHandle {
    /// Subscribe to a gossipsub topic
    pub async fn subscribe(&self, topic: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(P2PCommand::Subscribe {
                topic: topic.to_string(),
                response: tx,
            })
            .map_err(|_| "P2P node offline")?;
        rx.await.map_err(|_| "Command failed")?
    }

    /// Unsubscribe from a gossipsub topic
    pub async fn unsubscribe(&self, topic: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(P2PCommand::Unsubscribe {
                topic: topic.to_string(),
                response: tx,
            })
            .map_err(|_| "P2P node offline")?;
        rx.await.map_err(|_| "Command failed")?
    }

    /// Publish a message to a topic
    pub async fn publish(
        &self,
        topic: &str,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(P2PCommand::Publish {
                topic: topic.to_string(),
                data,
                response: tx,
            })
            .map_err(|_| "P2P node offline")?;
        rx.await.map_err(|_| "Command failed")?
    }

    /// Send a request to a peer
    pub async fn send_request(
        &self,
        peer_id: LibP2PPeerId,
        request: QuDagRequest,
    ) -> Result<QuDagResponse, Box<dyn Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(P2PCommand::SendRequest {
                peer_id,
                request,
                response: tx,
            })
            .map_err(|_| "P2P node offline")?;
        rx.await.map_err(|_| "Command failed")?
    }

    /// Dial a peer
    pub async fn dial(&self, addr: Multiaddr) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(P2PCommand::Dial { addr, response: tx })
            .map_err(|_| "P2P node offline")?;
        rx.await.map_err(|_| "Command failed")?
    }

    /// Get connected peers
    pub async fn connected_peers(&self) -> Vec<LibP2PPeerId> {
        let (tx, rx) = oneshot::channel();
        if self
            .command_tx
            .send(P2PCommand::GetConnectedPeers { response: tx })
            .is_ok()
        {
            rx.await.unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// Get local peer ID
    pub async fn local_peer_id(&self) -> LibP2PPeerId {
        let (tx, rx) = oneshot::channel();
        if self
            .command_tx
            .send(P2PCommand::GetLocalPeerId { response: tx })
            .is_ok()
        {
            rx.await.unwrap_or_else(|_| LibP2PPeerId::random())
        } else {
            LibP2PPeerId::random()
        }
    }

    /// Get listeners
    pub async fn listeners(&self) -> Vec<Multiaddr> {
        let (tx, rx) = oneshot::channel();
        if self
            .command_tx
            .send(P2PCommand::GetListeners { response: tx })
            .is_ok()
        {
            rx.await.unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// Get the next network event
    pub async fn next_event(&self) -> Option<P2PEvent> {
        let mut event_rx = self.event_rx.lock().await;
        event_rx.recv().await
    }
}

impl P2PNode {
    /// Creates a new P2P network node with the given configuration
    /// Returns the node and a handle for sending commands
    pub async fn new(config: NetworkConfig) -> Result<(Self, P2PHandle), Box<dyn Error>> {
        // Generate node identity
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = LibP2PPeerId::from(local_key.public());

        info!("Local peer ID: {}", local_peer_id);

        // Build the transport
        let transport = build_transport(&local_key, &config)?;

        // Set up Kademlia DHT
        let store = MemoryStore::new(local_peer_id);
        let mut kad_config = kad::Config::default();
        kad_config.set_replication_factor(
            std::num::NonZeroUsize::new(config.kad_replication_factor)
                .expect("Replication factor must be > 0"),
        );
        let kademlia = kad::Behaviour::with_config(local_peer_id, store, kad_config);

        // Set up Gossipsub
        let gossipsub_config = config.gossipsub_config.clone().unwrap_or_else(|| {
            GossipsubConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(ValidationMode::Strict)
                .build()
                .expect("Valid gossipsub config")
        });

        let gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )?;

        // Set up MDNS
        let mdns = if config.enable_mdns {
            Toggle::from(Some(mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                local_peer_id,
            )?))
        } else {
            Toggle::from(None)
        };

        // Set up other protocols
        let ping = ping::Behaviour::new(ping::Config::new());
        let identify = identify::Behaviour::new(identify::Config::new(
            "/qudag/1.0.0".to_string(),
            local_key.public(),
        ));

        let relay = relay::Behaviour::new(local_peer_id, Default::default());
        let dcutr = dcutr::Behaviour::new(local_peer_id);

        // Set up request-response protocol
        let protocols = std::iter::once((
            StreamProtocol::new("/qudag/req/1.0.0"),
            ProtocolSupport::Full,
        ));
        let request_response =
            request_response::cbor::Behaviour::new(protocols, request_response::Config::default());

        // Create the network behaviour
        let behaviour = NetworkBehaviourImpl {
            kademlia,
            gossipsub,
            mdns,
            ping,
            identify,
            relay,
            dcutr,
            request_response,
        };

        // Build the swarm
        let swarm = libp2p::Swarm::new(
            transport,
            behaviour,
            local_peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );

        // Set up channels and state
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (router_tx, _) = mpsc::channel(1024);
        let router = Router::new(router_tx);

        // Initialize traffic obfuscation
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&config.obfuscation_key));

        // Initialize metrics if enabled
        let metrics = if std::env::var("QUDAG_METRICS").is_ok() {
            Some(()) // TODO: Initialize proper metrics
        } else {
            None
        };

        // Create the handle
        let handle = P2PHandle {
            command_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
        };

        // Initialize message chunker (disabled for initial release)
        // let chunker_config = ChunkerConfig {
        //     max_chunk_size: 65536, // 64KB chunks
        //     enable_compression: true,
        //     compression_threshold: 1024, // Compress messages larger than 1KB
        //     ..Default::default()
        // };
        // let message_chunker = MessageChunker::new(chunker_config);

        let node = Self {
            local_peer_id,
            swarm,
            router,
            cipher,
            event_tx,
            command_rx,
            connected_peers: HashSet::new(),
            pending_requests: HashMap::new(),
            metrics,
            config,
            // message_chunker,
        };

        Ok((node, handle))
    }

    /// Starts the network node and begins listening on configured addresses
    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        // Listen on all configured addresses
        for addr_str in &self.config.listen_addrs {
            let addr: Multiaddr = addr_str.parse()?;
            self.swarm.listen_on(addr)?;
        }

        // Add bootstrap peers to Kademlia
        for peer_addr_str in &self.config.bootstrap_peers {
            let peer_addr: Multiaddr = peer_addr_str.parse()?;
            if let Some(peer_id) = extract_peer_id(&peer_addr) {
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, peer_addr);
            }
        }

        // Bootstrap Kademlia
        if let Err(e) = self.swarm.behaviour_mut().kademlia.bootstrap() {
            warn!("Kademlia bootstrap failed: {}", e);
        }

        info!("P2P node started");
        Ok(())
    }

    /// Main event loop for the P2P node
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            tokio::select! {
                swarm_event = self.swarm.next() => {
                    if let Some(event) = swarm_event {
                        self.handle_swarm_event(event).await?;
                    }
                }
                command = self.command_rx.recv() => {
                    if let Some(cmd) = command {
                        self.handle_command(cmd).await;
                    } else {
                        // Command channel closed, exit loop
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle swarm events
    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<NetworkBehaviourEvent>,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {}", address);
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                endpoint,
                num_established,
                ..
            } => {
                info!(
                    "Connection established with {} at {} ({} total connections)",
                    peer_id,
                    endpoint.get_remote_address(),
                    num_established
                );
                self.connected_peers.insert(peer_id);
                self.event_tx.send(P2PEvent::PeerConnected(peer_id))?;

                // Update router
                if let Ok(socket_addr) = endpoint.get_remote_address().to_string().parse() {
                    self.router
                        .add_discovered_peer(
                            peer_id,
                            crate::discovery::DiscoveredPeer::new(
                                peer_id,
                                socket_addr,
                                crate::discovery::DiscoveryMethod::Kademlia,
                            ),
                        )
                        .await;
                }
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                num_established,
                ..
            } => {
                info!(
                    "Connection closed with {} ({} remaining connections)",
                    peer_id, num_established
                );
                if num_established == 0 {
                    self.connected_peers.remove(&peer_id);
                    self.event_tx.send(P2PEvent::PeerDisconnected(peer_id))?;

                    // Update router
                    self.router.remove_discovered_peer(peer_id).await;
                }
            }
            SwarmEvent::Behaviour(behaviour_event) => {
                self.handle_behaviour_event(behaviour_event).await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle behaviour events
    async fn handle_behaviour_event(
        &mut self,
        event: NetworkBehaviourEvent,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            NetworkBehaviourEvent::Kademlia(kad_event) => {
                self.handle_kademlia_event(kad_event).await?;
            }
            NetworkBehaviourEvent::Gossipsub(gossipsub_event) => {
                self.handle_gossipsub_event(gossipsub_event).await?;
            }
            NetworkBehaviourEvent::Mdns(mdns_event) => {
                self.handle_mdns_event(mdns_event).await?;
            }
            NetworkBehaviourEvent::Ping(ping_event) => {
                self.handle_ping_event(ping_event).await?;
            }
            NetworkBehaviourEvent::Identify(identify_event) => {
                self.handle_identify_event(identify_event).await?;
            }
            NetworkBehaviourEvent::RequestResponse(req_res_event) => {
                self.handle_request_response_event(req_res_event).await?;
            }
            NetworkBehaviourEvent::Relay(relay_event) => {
                self.handle_relay_event(relay_event).await?;
            }
            NetworkBehaviourEvent::Dcutr(dcutr_event) => {
                self.handle_dcutr_event(dcutr_event).await?;
            }
        }
        Ok(())
    }

    /// Handle Kademlia events
    async fn handle_kademlia_event(&mut self, event: kad::Event) -> Result<(), Box<dyn Error>> {
        match event {
            kad::Event::RoutingUpdated {
                peer, addresses, ..
            } => {
                debug!("Kademlia routing updated for peer {}", peer);
                for addr in addresses.iter() {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer, addr.clone());
                }
                self.event_tx.send(P2PEvent::RoutingTableUpdated)?;
            }
            kad::Event::UnroutablePeer { peer } => {
                warn!("Peer {} is unroutable", peer);
            }
            kad::Event::InboundRequest { request } => {
                debug!("Kademlia inbound request: {:?}", request);
            }
            kad::Event::OutboundQueryProgressed { result, .. } => match result {
                QueryResult::GetClosestPeers(result) => match result {
                    Ok(ok) => {
                        for peer in ok.peers {
                            debug!("Found closest peer: {}", peer);
                            self.event_tx.send(P2PEvent::PeerDiscovered(peer))?;
                        }
                    }
                    Err(e) => warn!("Get closest peers error: {:?}", e),
                },
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    /// Handle Gossipsub events
    async fn handle_gossipsub_event(
        &mut self,
        event: gossipsub::Event,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            gossipsub::Event::Message {
                propagation_source,
                message,
                ..
            } => {
                let topic = message.topic.to_string();
                let data = message.data;

                // Deobfuscate if needed
                let decrypted_data = match self.deobfuscate_traffic(&data) {
                    Ok(d) => d,
                    Err(_) => data, // Assume not obfuscated
                };

                self.event_tx.send(P2PEvent::MessageReceived {
                    peer_id: propagation_source,
                    topic,
                    data: decrypted_data,
                })?;
            }
            gossipsub::Event::Subscribed { peer_id, topic } => {
                debug!("Peer {} subscribed to topic {}", peer_id, topic);
            }
            gossipsub::Event::Unsubscribed { peer_id, topic } => {
                debug!("Peer {} unsubscribed from topic {}", peer_id, topic);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle MDNS events
    async fn handle_mdns_event(&mut self, event: mdns::Event) -> Result<(), Box<dyn Error>> {
        match event {
            mdns::Event::Discovered(peers) => {
                for (peer_id, addr) in peers {
                    debug!("MDNS discovered peer {} at {}", peer_id, addr);
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr);
                    self.event_tx.send(P2PEvent::PeerDiscovered(peer_id))?;
                }
            }
            mdns::Event::Expired(peers) => {
                for (peer_id, _) in peers {
                    debug!("MDNS peer expired: {}", peer_id);
                }
            }
        }
        Ok(())
    }

    /// Handle ping events
    async fn handle_ping_event(&mut self, event: ping::Event) -> Result<(), Box<dyn Error>> {
        match event.result {
            Ok(duration) => {
                debug!("Ping to {} successful: {:?}", event.peer, duration);
            }
            Err(e) => {
                debug!("Ping to {} failed: {}", event.peer, e);
            }
        }
        Ok(())
    }

    /// Handle identify events
    async fn handle_identify_event(
        &mut self,
        event: identify::Event,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            identify::Event::Received { peer_id, info } => {
                debug!(
                    "Identified peer {}: protocols={:?}, agent={}",
                    peer_id, info.protocols, info.agent_version
                );

                // Add observed addresses to Kademlia
                for addr in info.listen_addrs {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr);
                }
            }
            identify::Event::Sent { .. } => {}
            identify::Event::Pushed { .. } => {}
            identify::Event::Error { peer_id, error } => {
                warn!("Identify error with {}: {}", peer_id, error);
            }
        }
        Ok(())
    }

    /// Handle relay events
    async fn handle_relay_event(&mut self, event: relay::Event) -> Result<(), Box<dyn Error>> {
        match event {
            relay::Event::ReservationReqAccepted {
                src_peer_id,
                renewed,
                ..
            } => {
                info!(
                    "Relay reservation accepted from peer {}: renewed={}",
                    src_peer_id, renewed
                );
            }
            relay::Event::ReservationReqDenied { src_peer_id, .. } => {
                warn!("Relay reservation denied by peer {}", src_peer_id);
            }
            relay::Event::ReservationTimedOut { src_peer_id, .. } => {
                warn!("Relay reservation timed out for peer {}", src_peer_id);
            }
            #[allow(deprecated)]
            relay::Event::CircuitReqAcceptFailed {
                src_peer_id,
                dst_peer_id,
                error,
            } => {
                warn!(
                    "Circuit request accept failed from {} to {}: {:?}",
                    src_peer_id, dst_peer_id, error
                );
            }
            relay::Event::CircuitReqDenied {
                src_peer_id,
                dst_peer_id,
                ..
            } => {
                warn!(
                    "Circuit request denied from {} to {}",
                    src_peer_id, dst_peer_id
                );
            }
            relay::Event::CircuitClosed {
                src_peer_id,
                dst_peer_id,
                error,
            } => {
                if let Some(error) = error {
                    warn!(
                        "Circuit closed between {} and {}: {:?}",
                        src_peer_id, dst_peer_id, error
                    );
                } else {
                    debug!("Circuit closed between {} and {}", src_peer_id, dst_peer_id);
                }
            }
            // Handle other relay events
            _ => {
                debug!("Unhandled relay event: {:?}", event);
            }
        }
        Ok(())
    }

    /// Handle DCUTR events
    async fn handle_dcutr_event(&mut self, event: dcutr::Event) -> Result<(), Box<dyn Error>> {
        match event {
            dcutr::Event {
                remote_peer_id,
                result,
            } => match result {
                Ok(connection_id) => {
                    info!(
                        "Direct connection upgrade succeeded with peer {} (connection: {:?})",
                        remote_peer_id, connection_id
                    );
                }
                Err(error) => {
                    warn!(
                        "Direct connection upgrade failed with {}: {:?}",
                        remote_peer_id, error
                    );
                }
            },
        }
        Ok(())
    }

    /// Handle request-response events
    async fn handle_request_response_event(
        &mut self,
        event: request_response::Event<QuDagRequest, QuDagResponse>,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            request_response::Event::Message { peer, message } => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    // Handle the request and prepare response
                    let response = QuDagResponse {
                        request_id: request.request_id.clone(),
                        payload: vec![], // TODO: Process request and generate actual response
                    };

                    // Send the response back directly
                    self.swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, response)
                        .map_err(|_| "Failed to send response")?;

                    // Also emit event for the application layer
                    let (tx, _rx) = oneshot::channel();
                    self.event_tx.send(P2PEvent::RequestReceived {
                        peer_id: peer,
                        request,
                        channel: tx,
                    })?;
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => {
                    if let Some(tx) = self.pending_requests.remove(&request_id.to_string()) {
                        let _ = tx.send(response);
                    }
                }
            },
            request_response::Event::OutboundFailure {
                peer,
                request_id,
                error,
            } => {
                warn!(
                    "Request to {} failed (id: {}): {:?}",
                    peer, request_id, error
                );
                self.pending_requests.remove(&request_id.to_string());
            }
            request_response::Event::InboundFailure {
                peer,
                request_id,
                error,
            } => {
                warn!(
                    "Inbound request from {} failed (id: {}): {:?}",
                    peer, request_id, error
                );
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle commands received from P2PHandle
    async fn handle_command(&mut self, command: P2PCommand) {
        match command {
            P2PCommand::Subscribe { topic, response } => {
                let result = self.subscribe_internal(&topic).await;
                let _ = response.send(result);
            }
            P2PCommand::Unsubscribe { topic, response } => {
                let result = self.unsubscribe_internal(&topic).await;
                let _ = response.send(result);
            }
            P2PCommand::Publish {
                topic,
                data,
                response,
            } => {
                let result = self.publish_internal(&topic, data).await;
                let _ = response.send(result);
            }
            P2PCommand::SendRequest {
                peer_id,
                request,
                response,
            } => {
                let result = self.send_request_internal(peer_id, request).await;
                let _ = response.send(result);
            }
            P2PCommand::Dial { addr, response } => {
                let result = self.dial_internal(addr).await;
                let _ = response.send(result);
            }
            P2PCommand::GetConnectedPeers { response } => {
                let peers = self.connected_peers.iter().copied().collect();
                let _ = response.send(peers);
            }
            P2PCommand::GetLocalPeerId { response } => {
                let _ = response.send(self.local_peer_id);
            }
            P2PCommand::GetListeners { response } => {
                let listeners = self.swarm.listeners().cloned().collect();
                let _ = response.send(listeners);
            }
        }
    }

    /// Internal subscribe method
    async fn subscribe_internal(
        &mut self,
        topic: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let topic = IdentTopic::new(topic);
        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&topic)
            .map_err(|e| format!("Subscribe error: {}", e))?;
        info!("Subscribed to topic: {}", topic);
        Ok(())
    }

    /// Internal unsubscribe method
    async fn unsubscribe_internal(
        &mut self,
        topic: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let topic = IdentTopic::new(topic);
        self.swarm
            .behaviour_mut()
            .gossipsub
            .unsubscribe(&topic)
            .map_err(|e| format!("Unsubscribe error: {}", e))?;
        info!("Unsubscribed from topic: {}", topic);
        Ok(())
    }

    /// Internal publish method
    async fn publish_internal(
        &mut self,
        topic: &str,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let topic = IdentTopic::new(topic);

        // Obfuscate traffic if configured
        let message_data = self
            .obfuscate_traffic(&data)
            .map_err(|e| format!("Obfuscation error: {}", e))?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic.clone(), message_data)
            .map_err(|e| format!("Publish error: {}", e))?;

        debug!("Published message to topic: {}", topic);
        Ok(())
    }

    /// Internal send request method with chunking support
    async fn send_request_internal(
        &mut self,
        peer_id: LibP2PPeerId,
        request: QuDagRequest,
    ) -> Result<QuDagResponse, Box<dyn Error + Send + Sync>> {
        let request_id = request.request_id.clone();

        // Check if message needs chunking
        let network_message = NetworkMessage {
            id: request.request_id.clone(),
            source: vec![0],      // Placeholder source
            destination: vec![0], // Placeholder destination
            payload: request.payload.clone(),
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        // Chunking disabled for initial release - send message directly
        // let chunks = self.message_chunker.chunk_message(&network_message).await
        //     .map_err(|e| format!("Chunking error: {:?}", e))?;

        // Send message directly without chunking
        let request = QuDagRequest {
            request_id: request_id.clone(),
            payload: bincode::serialize(&network_message)
                .map_err(|e| format!("Serialization error: {}", e))?,
        };

        // Setup response handling
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(request_id.clone(), tx);

        self.swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, request);

        // Wait for response with timeout
        match tokio::time::timeout(self.config.timeout, rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err("Response channel closed".into()),
            Err(_) => {
                self.pending_requests.remove(&request_id);
                Err("Request timeout".into())
            }
        }
    }

    /// Internal dial method
    async fn dial_internal(
        &mut self,
        peer_addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.swarm
            .dial(peer_addr)
            .map_err(|e| format!("Dial error: {}", e))?;
        Ok(())
    }

    /// Obfuscates traffic using ChaCha20-Poly1305
    fn obfuscate_traffic(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut nonce = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce);
        let nonce = Nonce::from_slice(&nonce);

        let mut encrypted = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|e| format!("Encryption error: {}", e))?;

        // Prepend nonce to encrypted data
        let mut result = nonce.to_vec();
        result.append(&mut encrypted);
        Ok(result)
    }

    /// Deobfuscates traffic using ChaCha20-Poly1305
    fn deobfuscate_traffic(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if data.len() < 12 {
            return Err("Data too short".into());
        }

        let nonce = Nonce::from_slice(&data[..12]);
        let encrypted = &data[12..];

        self.cipher
            .decrypt(nonce, encrypted)
            .map_err(|e| format!("Decryption error: {}", e).into())
    }
}

/// Build the transport layer with multiple protocol support
fn build_transport(
    local_key: &Keypair,
    config: &NetworkConfig,
) -> Result<Boxed<(LibP2PPeerId, StreamMuxerBox)>, Box<dyn Error>> {
    let noise = noise::Config::new(local_key)?;

    let yamux_config = yamux::Config::default();

    // Build base TCP transport
    let tcp = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true));

    // Memory transport for testing
    let memory = MemoryTransport::default();

    // Combine transports
    let base_transport = tcp.or_transport(memory);

    // Add WebSocket support if enabled
    let transport: Boxed<(LibP2PPeerId, StreamMuxerBox)> = if config.enable_websocket {
        let ws = websocket::WsConfig::new(tcp::tokio::Transport::new(
            tcp::Config::default().nodelay(true),
        ));
        base_transport
            .or_transport(ws)
            .upgrade(upgrade::Version::V1)
            .authenticate(noise)
            .multiplex(yamux_config)
            .timeout(Duration::from_secs(20))
            .boxed()
    } else {
        base_transport
            .upgrade(upgrade::Version::V1)
            .authenticate(noise)
            .multiplex(yamux_config)
            .timeout(Duration::from_secs(20))
            .boxed()
    };

    Ok(transport)
}

/// Extract peer ID from multiaddr if present
fn extract_peer_id(addr: &Multiaddr) -> Option<LibP2PPeerId> {
    addr.iter().find_map(|p| match p {
        Protocol::P2p(peer_id) => Some(peer_id),
        _ => None,
    })
}

/// Type alias for stream muxer
type StreamMuxerBox = libp2p::core::muxing::StreamMuxerBox;

/// Type aliases for missing libp2p types in 0.53
#[allow(dead_code)]
type TransactionId = [u8; 12];
#[allow(dead_code)]
type Message = Vec<u8>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_creation() {
        let config = NetworkConfig::default();
        let (_node, handle) = P2PNode::new(config).await.unwrap();
        let peer_id = handle.local_peer_id().await;
        assert!(!peer_id.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_traffic_obfuscation() {
        let config = NetworkConfig::default();
        let (node, _handle) = P2PNode::new(config).await.unwrap();

        let test_data = b"test message";
        let obfuscated = node.obfuscate_traffic(test_data).unwrap();
        let deobfuscated = node.deobfuscate_traffic(&obfuscated).unwrap();

        assert_eq!(test_data.to_vec(), deobfuscated);
    }

    #[tokio::test]
    async fn test_node_start() {
        let mut config = NetworkConfig::default();
        config.listen_addrs = vec!["/ip4/127.0.0.1/tcp/0".to_string()];
        config.enable_mdns = false; // Disable MDNS for tests

        let (mut node, handle) = P2PNode::new(config).await.unwrap();
        node.start().await.unwrap();

        // Give it a moment to bind
        tokio::time::sleep(Duration::from_millis(100)).await;

        let listeners = handle.listeners().await;
        assert!(!listeners.is_empty());
    }

    #[tokio::test]
    async fn test_pubsub() {
        let config = NetworkConfig::default();
        let (_node, handle) = P2PNode::new(config).await.unwrap();

        let topic = "test-topic";
        handle.subscribe(topic).await.unwrap();

        let test_data = vec![1, 2, 3, 4, 5];
        handle.publish(topic, test_data).await.unwrap();
    }
}
