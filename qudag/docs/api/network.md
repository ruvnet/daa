# Network Module API

The `qudag_network` module provides P2P networking layer with anonymous routing, traffic obfuscation, and quantum-resistant transport security for the QuDAG protocol.

## P2P Node Implementation

### P2PNode

Main P2P network node with libp2p integration and traffic obfuscation.

```rust
pub struct P2PNode {
    local_peer_id: PeerId,
    // private fields
}

impl P2PNode {
    pub async fn new(config: NetworkConfig) -> Result<Self, NetworkError>;
    pub async fn start(&mut self) -> Result<(), NetworkError>;
    pub async fn stop(&mut self) -> Result<(), NetworkError>;
    pub async fn send_message(&mut self, peer: PeerId, data: Vec<u8>) -> Result<(), NetworkError>;
    pub async fn broadcast(&mut self, data: Vec<u8>) -> Result<(), NetworkError>;
    pub fn get_local_peer_id(&self) -> PeerId;
    pub fn get_connected_peers(&self) -> Vec<PeerId>;
}
```

### NetworkConfig

Configuration for P2P network node with security settings.

```rust
pub struct NetworkConfig {
    pub listen_addr: String,
    pub bootstrap_peers: Vec<String>,
    pub timeout: Duration,
    pub max_connections: usize,
    pub obfuscation_key: [u8; 32],
}

impl Default for NetworkConfig {
    fn default() -> Self {
        // Returns secure default configuration
    }
}
```

## Core Types

### MessageHandler

Manages message sending and receiving with high-throughput queues.

```rust
pub struct MessageHandler {
    // private fields
}

impl MessageHandler {
    pub fn new() -> Self;
    pub async fn send(&self, msg: Message) -> Result<(), NetworkError>;
    pub async fn receive(&self) -> Result<Message, NetworkError>;
    pub fn get_stats(&self) -> Arc<RwLock<QueueStats>>;
}
```

### Message

Network message with routing information.

```rust
pub struct Message {
    // private fields
}

impl Message {
    pub fn new(content: Vec<u8>, destination: PeerId, route: Route) -> Self;
    pub fn encrypt(self) -> Self;
    pub fn decrypt(self) -> Result<Self, NetworkError>;
    pub fn content(&self) -> &[u8];
    pub fn route(&self) -> &Route;
    pub fn is_encrypted(&self) -> bool;
}
```

### Route

Defines message routing path and anonymity settings.

```rust
pub struct Route {
    // private fields
}

impl Route {
    pub fn new() -> Self;
    pub fn direct() -> Self;
    pub fn add_hop(mut self, peer: PeerId) -> Self;
    pub fn next_hop(&self) -> Option<&PeerId>;
    pub fn is_anonymous(&self) -> bool;
    pub fn reveals_sender(&self) -> bool;
}
```

### PeerId

Unique identifier for network nodes.

```rust
pub struct PeerId(Vec<u8>);

impl PeerId {
    pub fn random() -> Self;
}
```

## Transport Layer

### Transport

Secure transport with quantum-resistant encryption and traffic obfuscation.

```rust
pub struct Transport {
    // private fields
}

impl Transport {
    pub fn new(config: TransportConfig) -> Result<Self, TransportError>;
    pub async fn connect(&self, addr: SocketAddr) -> Result<Connection, TransportError>;
    pub async fn listen(&self, addr: SocketAddr) -> Result<Listener, TransportError>;
    pub fn obfuscate_traffic(&self, data: &[u8]) -> Vec<u8>;
    pub fn deobfuscate_traffic(&self, data: &[u8]) -> Result<Vec<u8>, TransportError>;
}
```

### Router

Anonymous routing with onion routing and traffic mixing.

```rust
pub struct Router {
    // private fields  
}

impl Router {
    pub fn new() -> Self;
    pub async fn route_message(&self, msg: Message, path: RoutePath) -> Result<(), RoutingError>;
    pub fn create_onion_route(&self, destination: PeerId, hops: Vec<PeerId>) -> Result<RoutePath, RoutingError>;
    pub fn add_routing_entry(&mut self, dest: PeerId, next_hop: PeerId);
    pub fn remove_routing_entry(&mut self, dest: &PeerId);
}
```

### RoutePath  

Defines routing path for anonymous message delivery.

```rust
pub struct RoutePath {
    pub hops: Vec<PeerId>,
    pub encrypted_layers: Vec<Vec<u8>>,
}

impl RoutePath {
    pub fn new(hops: Vec<PeerId>) -> Self;
    pub fn peel_layer(&mut self) -> Result<PeerId, RoutingError>;
    pub fn is_final_hop(&self) -> bool;
}
```

## Core Traits

### NetworkNode

Base trait for network nodes.

```rust
pub trait NetworkNode: Send + Sync + 'static {
    fn start(&self) -> Pin<Box<dyn Future<Output = Result<(), NetworkError>> + Send>>;
    fn stop(&self) -> Pin<Box<dyn Future<Output = Result<(), NetworkError>> + Send>>;
    fn metrics(&self) -> Pin<Box<dyn Future<Output = NetworkMetrics> + Send>>;
    fn status(&self) -> ConnectionStatus;
}
```

### PeerDiscovery

Handles peer discovery and management.

```rust
pub trait PeerDiscovery: Send + Sync + 'static {
    fn add_peer(&self, peer: PeerId) -> Pin<Box<dyn Future<Output = Result<(), NetworkError>> + Send>>;
    fn remove_peer(&self, peer: &PeerId) -> Pin<Box<dyn Future<Output = Result<(), NetworkError>> + Send>>;
    fn get_peers(&self) -> Pin<Box<dyn Future<Output = Vec<PeerId>> + Send>>;
    fn find_peers(&self, service: &str) -> Pin<Box<dyn Future<Output = Vec<PeerId>> + Send>>;
}
```

### AnonymousRouting

Provides anonymous routing capabilities.

```rust
pub trait AnonymousRouting: Send + Sync + 'static {
    fn create_route(&self, destination: PeerId, hops: usize) 
        -> Pin<Box<dyn Future<Output = Result<Route, NetworkError>> + Send>>;
    fn next_hop(&self, route: &Route) -> Option<PeerId>;
    fn validate_route(&self, route: &Route) -> bool;
    fn update_routing_table(&self, routes: Vec<Route>) 
        -> Pin<Box<dyn Future<Output = Result<(), NetworkError>> + Send>>;
}
```

## Error Types

### NetworkError

```rust
pub enum NetworkError {
    InvalidRoute,
    MessageTooLarge,
    EncryptionError(String),
    ConnectionError(String),
    Internal(String),
}
```

## Example Usage

### Setting Up P2P Node

```rust
use qudag_network::{P2PNode, NetworkConfig, NetworkError};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), NetworkError> {
    // Configure P2P node
    let config = NetworkConfig {
        listen_addr: "/ip4/0.0.0.0/tcp/4001".to_string(),
        bootstrap_peers: vec![
            "/ip4/192.168.1.100/tcp/4001/p2p/12D3KooW...".to_string()
        ],
        timeout: Duration::from_secs(30),
        max_connections: 100,
        obfuscation_key: [0u8; 32], // Use secure random key in production
    };
    
    // Create and start P2P node
    let mut node = P2PNode::new(config).await?;
    node.start().await?;
    
    println!("P2P node started with ID: {:?}", node.get_local_peer_id());
    
    // Send message to peer
    let peer_id = /* get peer ID */;
    let message = b"Hello, P2P world!".to_vec();
    node.send_message(peer_id, message).await?;
    
    // Broadcast to all peers
    node.broadcast(b"Broadcast message".to_vec()).await?;
    
    Ok(())
}
```

### Anonymous Routing with Onion Routing

```rust
use qudag_network::{Router, RoutePath, Message, PeerId};

async fn setup_anonymous_communication() -> Result<(), NetworkError> {
    let router = Router::new();
    
    // Create anonymous route with multiple hops
    let destination = PeerId::random();
    let hops = vec![
        PeerId::random(), // Hop 1
        PeerId::random(), // Hop 2  
        PeerId::random(), // Hop 3
        destination,      // Final destination
    ];
    
    let route_path = router.create_onion_route(destination, hops)?;
    
    // Send message through anonymous route
    let message = Message::new(
        b"Anonymous message".to_vec(),
        destination,
        Route::from_path(route_path)
    );
    
    router.route_message(message, route_path).await?;
    
    Ok(())
}
```

### Traffic Obfuscation

```rust
use qudag_network::{Transport, TransportConfig};

async fn setup_obfuscated_transport() -> Result<(), NetworkError> {
    let config = TransportConfig {
        obfuscation_enabled: true,
        obfuscation_key: secure_random_key(),
        quantum_resistant: true,
    };
    
    let transport = Transport::new(config)?;
    
    // Connect with traffic obfuscation
    let connection = transport.connect("127.0.0.1:8080".parse()?).await?;
    
    // Send obfuscated data
    let sensitive_data = b"Sensitive protocol data";
    let obfuscated = transport.obfuscate_traffic(sensitive_data);
    connection.send(obfuscated).await?;
    
    // Receive and deobfuscate
    let received = connection.receive().await?;
    let deobfuscated = transport.deobfuscate_traffic(&received)?;
    
    println!("Deobfuscated: {:?}", deobfuscated);
    
    Ok(())
}
```

### Basic Message Handling

```rust
use qudag_network::{MessageHandler, Message, PeerId, Route};

#[tokio::main]
async fn main() -> Result<(), NetworkError> {
    // Create message handler
    let handler = MessageHandler::new();
    
    // Create and send a message
    let dest = PeerId::random();
    let route = Route::new().add_hop(PeerId::random());
    let msg = Message::new(b"Hello".to_vec(), dest, route).encrypt();
    
    handler.send(msg).await?;
    
    // Receive and process messages
    if let Ok(msg) = handler.receive().await {
        if msg.is_encrypted() {
            let decrypted = msg.decrypt()?;
            println!("Received: {:?}", decrypted.content());
        }
    }
    
    Ok(())
}
```

### Anonymous Routing

```rust
use qudag_network::{AnonymousRouting, PeerId, Route};

async fn setup_anonymous_route(
    router: &impl AnonymousRouting,
    dest: PeerId
) -> Result<Route, NetworkError> {
    // Create route with 3 hops for anonymity
    let route = router.create_route(dest, 3).await?;
    
    // Validate the route
    if !router.validate_route(&route) {
        return Err(NetworkError::InvalidRoute);
    }
    
    Ok(route)
}
```

### Peer Discovery

```rust
use qudag_network::{PeerDiscovery, PeerId};

async fn discover_service_peers(
    discovery: &impl PeerDiscovery,
    service: &str
) -> Vec<PeerId> {
    // Find peers providing specific service
    let peers = discovery.find_peers(service).await;
    
    // Add new peers to network
    for peer in &peers {
        if let Err(e) = discovery.add_peer(peer.clone()).await {
            eprintln!("Failed to add peer: {}", e);
        }
    }
    
    peers
}
```

## Best Practices

1. **Message Handling**
   - Always encrypt sensitive messages
   - Handle message size limits
   - Implement proper error handling
   - Monitor queue performance

2. **Anonymous Routing**
   - Use multiple hops for better anonymity
   - Validate routes before use
   - Implement route redundancy
   - Regular routing table updates

3. **Peer Management**
   - Regular peer discovery
   - Proper peer validation
   - Maintain optimal peer count
   - Handle peer disconnections

## Security Considerations

1. **Message Privacy**
   - All messages should be encrypted
   - Use anonymous routes for sensitive data
   - Clear message content after processing
   - Avoid logging sensitive data

2. **Network Security**
   - Validate peer connections
   - Monitor for malicious behavior
   - Implement rate limiting
   - Regular security audits

3. **Anonymity Protection**
   - Use sufficient routing hops
   - Avoid sender/receiver correlation
   - Implement mixing strategies
   - Regular anonymity analysis