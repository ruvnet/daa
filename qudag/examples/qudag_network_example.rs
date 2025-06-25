//! Example demonstrating QuDAG network setup with all features.
//!
//! This example shows how to:
//! - Setup quantum-resistant P2P overlay
//! - Configure Kademlia DHT for peer discovery
//! - Enable onion routing for privacy
//! - Setup DAG-based consensus (QR-Avalanche)
//! - Use .dark address resolution
//! - Enable WebRTC for browser nodes

use qudag_network::{
    NetworkManager, NetworkConfig, P2PNode, P2PNetworkConfig,
    KademliaDHT, DHTConfig, BootstrapConfig, PeerScoringConfig,
    OnionRouter, TrafficAnalysisConfig,
    DarkResolver, DarkDomainRecord,
    WebRTCConfig, WebRTCTransport, create_webrtc_transport,
    DagConsensusNetwork, ConsensusNetworkConfig, ConsensusNetworkEvent,
    create_dag_consensus_network,
};
use qudag_dag::{Vertex, VertexId, ConsensusConfig as DagConsensusConfig};
use libp2p::PeerId as LibP2PPeerId;
use std::collections::HashSet;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting QuDAG network example");

    // 1. Create network configuration
    let network_config = NetworkConfig {
        max_connections: 100,
        connection_timeout: Duration::from_secs(30),
        discovery_interval: Duration::from_secs(60),
        bootstrap_peers: vec![
            "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWExample1".to_string(),
            "/ip4/127.0.0.1/tcp/4002/p2p/12D3KooWExample2".to_string(),
        ],
        enable_dht: true,
        quantum_resistant: true,
        enable_nat_traversal: true,
        nat_traversal_config: None,
    };

    // 2. Initialize network manager
    let mut network_manager = NetworkManager::with_config(network_config);
    network_manager.initialize().await?;

    let local_peer_id = network_manager.local_peer_id()
        .ok_or("Failed to get local peer ID")?;
    
    info!("Local peer ID: {:?}", local_peer_id);

    // 3. Setup Kademlia DHT for peer discovery
    let dht_config = DHTConfig::default();
    let bootstrap_config = BootstrapConfig {
        nodes: vec![
            (LibP2PPeerId::random(), "127.0.0.1:4001".parse()?),
            (LibP2PPeerId::random(), "127.0.0.1:4002".parse()?),
        ],
        timeout: Duration::from_secs(30),
        min_connections: 2,
        periodic_bootstrap: true,
        bootstrap_interval: Duration::from_secs(3600),
    };
    let scoring_config = PeerScoringConfig::default();

    let mut kademlia_dht = KademliaDHT::new(
        local_peer_id,
        dht_config,
        bootstrap_config,
        DagConsensusConfig::default(),
        scoring_config,
    );

    // Bootstrap the DHT
    info!("Bootstrapping Kademlia DHT...");
    kademlia_dht.bootstrap().await?;

    // 4. Setup onion routing for privacy
    let mut onion_router = OnionRouter::new();
    onion_router.set_traffic_analysis_config(TrafficAnalysisConfig {
        enable_timing_obfuscation: true,
        enable_dummy_traffic: true,
        dummy_traffic_rate: 10, // 10 dummy messages per minute
        enable_traffic_mixing: true,
        mixing_delay: Duration::from_millis(500),
    });

    // Create a 3-hop circuit
    let circuit = onion_router.create_circuit(vec![
        LibP2PPeerId::random(),
        LibP2PPeerId::random(),
        LibP2PPeerId::random(),
    ]).await?;
    
    info!("Created onion circuit with {} hops", circuit.hops.len());

    // 5. Setup dark address resolution
    let mut dark_resolver = DarkResolver::new();
    
    // Register a .dark domain
    let dark_domain = DarkDomainRecord {
        domain: "mynode.dark".to_string(),
        address: local_peer_id.to_string(),
        ttl: 3600,
        quantum_fingerprint: vec![0x01, 0x02, 0x03], // Example fingerprint
    };
    
    dark_resolver.register_domain(dark_domain.clone()).await?;
    info!("Registered dark domain: {}", dark_domain.domain);

    // 6. Setup WebRTC for browser nodes
    let webrtc_config = WebRTCConfig {
        stun_servers: vec![
            "stun:stun.l.google.com:19302".to_string(),
            "stun:stun1.l.google.com:19302".to_string(),
        ],
        turn_servers: vec![],
        max_message_size: 16 * 1024 * 1024,
        ordered: true,
        max_retransmits: None,
        verify_fingerprint: true,
        ice_gathering_timeout: Duration::from_secs(10),
        signaling_server: Some("wss://signaling.example.com".to_string()),
    };

    let webrtc_transport = create_webrtc_transport(webrtc_config);
    info!("WebRTC transport initialized for browser node support");

    // 7. Setup DAG-based consensus (QR-Avalanche)
    let consensus_config = ConsensusNetworkConfig {
        query_timeout: Duration::from_secs(5),
        sync_batch_size: 100,
        max_concurrent_queries: 50,
        enable_quantum_channels: true,
        min_peer_reputation: 0.5,
        partition_detection_threshold: Duration::from_secs(60),
        dag_config: DagConsensusConfig {
            query_sample_size: 10,
            finality_threshold: 0.8,
            finality_timeout: Duration::from_secs(5),
            confirmation_depth: 3,
        },
    };

    let peer_id = qudag_network::PeerId::from_bytes(local_peer_id.to_bytes()[..32].try_into()?);
    let mut dag_consensus = create_dag_consensus_network(peer_id, consensus_config);
    
    // Start consensus network
    dag_consensus.start().await?;
    info!("DAG consensus network started with QR-Avalanche");

    // 8. Example: Submit a vertex to the DAG
    let vertex_id = VertexId::new();
    let vertex = Vertex::new(
        vertex_id.clone(),
        b"Example transaction data".to_vec(),
        HashSet::new(), // No parents for genesis vertex
    );

    dag_consensus.submit_vertex(vertex).await?;
    info!("Submitted vertex to DAG consensus");

    // 9. Setup P2P node with all features
    let p2p_config = P2PNetworkConfig {
        listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".parse()?],
        bootstrap_peers: vec![],
        max_connections: 100,
        connection_timeout: Duration::from_secs(30),
        enable_mdns: true,
        enable_relay: true,
        enable_dcutr: true,
        timeout: Duration::from_secs(60),
    };

    let (mut p2p_node, p2p_handle) = P2PNode::new(p2p_config).await?;
    
    // Set P2P handle for DAG consensus
    dag_consensus.set_p2p_handle(p2p_handle.clone());

    // 10. Start P2P node
    let (event_tx, mut event_rx) = mpsc::channel(1024);
    tokio::spawn(async move {
        if let Err(e) = p2p_node.run().await {
            error!("P2P node error: {}", e);
        }
    });

    // 11. Handle network events
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                ConsensusNetworkEvent::VertexReceived(vertex) => {
                    info!("Received vertex: {:?}", vertex.id);
                }
                ConsensusNetworkEvent::VertexFinalized(vertex_id) => {
                    info!("Vertex finalized: {:?}", vertex_id);
                }
                ConsensusNetworkEvent::SyncCompleted { vertices_received } => {
                    info!("Sync completed, received {} vertices", vertices_received);
                }
                ConsensusNetworkEvent::QueryTimeout { vertex_id } => {
                    warn!("Query timeout for vertex: {:?}", vertex_id);
                }
                ConsensusNetworkEvent::PartitionDetected { affected_peers } => {
                    warn!("Network partition detected, affected peers: {:?}", affected_peers);
                }
            }
        }
    });

    // 12. Example operations
    
    // Connect to a peer
    match network_manager.connect_peer("/ip4/127.0.0.1/tcp/4001").await {
        Ok(peer_id) => info!("Connected to peer: {:?}", peer_id),
        Err(e) => warn!("Failed to connect to peer: {}", e),
    }

    // Store content in DHT
    let content_key = b"example_content";
    let content_value = b"This is example content stored in the DHT";
    kademlia_dht.store_record(content_key.to_vec(), content_value.to_vec()).await?;
    info!("Stored content in DHT");

    // Resolve a dark address
    if let Some(resolved) = dark_resolver.resolve("mynode.dark").await? {
        info!("Resolved dark address: {} -> {}", resolved.domain, resolved.address);
    }

    // Get consensus statistics
    let consensus_stats = dag_consensus.get_stats().await;
    info!("Consensus stats: {:?}", consensus_stats);

    // Get network statistics
    let network_stats = network_manager.get_network_stats().await;
    info!("Network stats: {:?}", network_stats);

    // Keep the example running
    info!("QuDAG network is running. Press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;

    // Cleanup
    info!("Shutting down QuDAG network...");
    network_manager.shutdown().await?;

    Ok(())
}

/// Example of creating a custom message handler
async fn handle_custom_message(data: Vec<u8>) {
    info!("Received custom message: {} bytes", data.len());
    // Process the message...
}

/// Example of establishing a quantum-secure channel
async fn establish_quantum_channel_example(
    dag_consensus: &DagConsensusNetwork,
    peer_id: &qudag_network::PeerId,
) -> Result<(), Box<dyn std::error::Error>> {
    dag_consensus.establish_quantum_channel(peer_id).await?;
    info!("Established quantum-secure channel with peer: {:?}", peer_id);
    Ok(())
}