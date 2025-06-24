# QuDAG Protocol Sequence Diagrams

This document provides detailed sequence diagrams for key protocol flows in the QuDAG system, illustrating the interactions between components during various operations.

## Table of Contents

1. [Node Initialization](#node-initialization)
2. [Message Processing](#message-processing)
3. [Consensus Flow](#consensus-flow)
4. [Anonymous Routing](#anonymous-routing)
5. [Dark Address Resolution](#dark-address-resolution)
6. [Peer Discovery](#peer-discovery)
7. [Error Handling](#error-handling)

## Node Initialization

### Complete Node Startup Sequence

```mermaid
sequenceDiagram
    participant Main as Main Process
    participant Config as Configuration
    participant Crypto as Crypto Module
    participant Network as Network Module
    participant DAG as DAG Module
    participant Protocol as Protocol Coordinator
    participant DHT as Kademlia DHT

    Main->>Config: Load configuration
    Config-->>Main: Configuration loaded

    Main->>Crypto: Initialize crypto system
    Crypto->>Crypto: Generate ML-KEM keypair
    Crypto->>Crypto: Generate ML-DSA keypair
    Crypto->>Crypto: Initialize BLAKE3 hasher
    Crypto-->>Main: Crypto initialized

    Main->>DAG: Initialize DAG consensus
    DAG->>DAG: Initialize QR-Avalanche
    DAG->>DAG: Create genesis vertex
    DAG-->>Main: DAG initialized

    Main->>Network: Initialize network layer
    Network->>Network: Setup P2P transport
    Network->>Network: Initialize onion router
    Network->>DHT: Connect to bootstrap nodes
    DHT-->>Network: Bootstrap connections established
    Network-->>Main: Network initialized

    Main->>Protocol: Start protocol coordinator
    Protocol->>Protocol: Initialize message handler
    Protocol->>Protocol: Start consensus engine
    Protocol->>Protocol: Begin peer discovery
    Protocol-->>Main: Protocol started

    Main->>Main: Node fully operational
```

### Cryptographic Key Initialization

```mermaid
sequenceDiagram
    participant App as Application
    participant KEM as ML-KEM Module
    participant DSA as ML-DSA Module
    participant Store as Key Store
    participant Memory as Secure Memory

    App->>KEM: Initialize ML-KEM-768
    KEM->>Memory: Allocate secure memory
    Memory-->>KEM: Aligned memory allocated
    KEM->>KEM: Generate entropy
    KEM->>KEM: Generate keypair
    KEM->>Store: Store public key
    KEM->>Memory: Store secret key (zeroize on drop)
    KEM-->>App: ML-KEM initialized

    App->>DSA: Initialize ML-DSA
    DSA->>Memory: Allocate secure memory
    Memory-->>DSA: Aligned memory allocated
    DSA->>DSA: Generate entropy
    DSA->>DSA: Generate signing keypair
    DSA->>Store: Store verify key
    DSA->>Memory: Store signing key (zeroize on drop)
    DSA-->>App: ML-DSA initialized

    App->>App: Cryptographic system ready
```

## Message Processing

### Complete Message Lifecycle

```mermaid
sequenceDiagram
    participant Sender as Sender Node
    participant SCrypto as Sender Crypto
    participant SDAG as Sender DAG
    participant SNetwork as Sender Network
    participant Router as Onion Router
    participant RNetwork as Receiver Network
    participant RDAG as Receiver DAG
    participant RCrypto as Receiver Crypto
    participant Receiver as Receiver Node

    Sender->>Sender: Create message payload
    Sender->>SCrypto: Sign message with ML-DSA
    SCrypto->>SCrypto: Compute BLAKE3 hash
    SCrypto->>SCrypto: Generate ML-DSA signature
    SCrypto-->>Sender: Message signed

    Sender->>SDAG: Create DAG vertex
    SDAG->>SDAG: Select parent vertices
    SDAG->>SDAG: Create vertex with parents
    SDAG->>SCrypto: Sign vertex
    SCrypto-->>SDAG: Vertex signed
    SDAG->>SDAG: Add to local DAG
    SDAG-->>Sender: Vertex added

    Sender->>SNetwork: Route message
    SNetwork->>Router: Create anonymous route
    Router->>Router: Build 3-7 hop circuit
    Router->>Router: Encrypt with ML-KEM layers
    Router-->>SNetwork: Route established

    SNetwork->>RNetwork: Send through onion route
    Note over SNetwork,RNetwork: Message passes through multiple hops
    RNetwork->>RNetwork: Decrypt onion layers
    RNetwork-->>Receiver: Message received

    Receiver->>RCrypto: Verify message signature
    RCrypto->>RCrypto: Verify ML-DSA signature
    RCrypto-->>Receiver: Signature valid

    Receiver->>RDAG: Add vertex to DAG
    RDAG->>RDAG: Validate vertex parents
    RDAG->>RDAG: Add to local DAG
    RDAG->>RDAG: Trigger consensus process
    RDAG-->>Receiver: Vertex processed

    Receiver->>Receiver: Message processing complete
```

### Quantum-Resistant Signature Verification

```mermaid
sequenceDiagram
    participant Node as Receiving Node
    participant Msg as Message Handler
    participant Crypto as Crypto Module
    participant DSA as ML-DSA Verifier
    participant Hash as BLAKE3 Hasher

    Node->>Msg: Receive message
    Msg->>Crypto: Verify signature
    Crypto->>Hash: Hash message content
    Hash-->>Crypto: Message hash
    Crypto->>DSA: Verify ML-DSA signature
    DSA->>DSA: Constant-time verification
    DSA-->>Crypto: Verification result
    
    alt Signature Valid
        Crypto-->>Msg: Signature verified
        Msg->>Node: Accept message
    else Signature Invalid
        Crypto-->>Msg: Verification failed
        Msg->>Node: Reject message
        Node->>Node: Log security event
    end
```

## Consensus Flow

### QR-Avalanche Consensus Process

```mermaid
sequenceDiagram
    participant Node as Local Node
    participant Consensus as QR-Avalanche
    participant Network as Network Layer
    participant Peer1 as Peer Node 1
    participant Peer2 as Peer Node 2
    participant PeerN as Peer Node N

    Node->>Consensus: Submit vertex for consensus
    Consensus->>Consensus: Initialize consensus round
    Consensus->>Network: Query random peer sample
    
    par Query Multiple Peers
        Network->>Peer1: Send consensus query
        Network->>Peer2: Send consensus query
        Network->>PeerN: Send consensus query
    end

    par Receive Peer Responses
        Peer1->>Network: Vote response
        Peer2->>Network: Vote response
        PeerN->>Network: Vote response
    end

    Network-->>Consensus: Aggregate votes
    Consensus->>Consensus: Calculate confidence
    
    alt Confidence > Threshold
        Consensus->>Consensus: Mark as preferred
        Consensus->>Consensus: Start finality timer
        
        loop Until Finality
            Consensus->>Network: Query peer confidence
            Network-->>Consensus: Peer responses
            Consensus->>Consensus: Update confidence
            
            alt Finality Achieved
                Consensus->>Consensus: Mark as finalized
                Consensus-->>Node: Vertex finalized
            else Timeout
                Consensus->>Consensus: Mark as rejected
                Consensus-->>Node: Consensus failed
            end
        end
    else Confidence < Threshold
        Consensus->>Consensus: Mark as rejected
        Consensus-->>Node: Vertex rejected
    end
```

### Concurrent Consensus Management

```mermaid
sequenceDiagram
    participant Coordinator as Consensus Coordinator
    participant Round1 as Consensus Round 1
    participant Round2 as Consensus Round 2
    participant RoundN as Consensus Round N
    participant State as Shared State
    participant Cleanup as Cleanup Manager

    Coordinator->>Round1: Start consensus (Vertex A)
    Round1->>State: Update round state
    
    Coordinator->>Round2: Start consensus (Vertex B)
    Round2->>State: Update round state
    
    Coordinator->>RoundN: Start consensus (Vertex N)
    RoundN->>State: Update round state

    par Concurrent Processing
        Round1->>Round1: Process consensus
        Round2->>Round2: Process consensus
        RoundN->>RoundN: Process consensus
    end

    alt Round 1 Completes
        Round1->>State: Update final result
        Round1->>Cleanup: Signal completion
        Cleanup->>Cleanup: Clean up round resources
    end

    alt Round 2 Completes
        Round2->>State: Update final result
        Round2->>Cleanup: Signal completion
        Cleanup->>Cleanup: Clean up round resources
    end

    Coordinator->>State: Query consensus results
    State-->>Coordinator: Current state
```

## Anonymous Routing

### Onion Route Creation and Message Transmission

```mermaid
sequenceDiagram
    participant Client as Client Node
    participant Router as Onion Router
    participant Hop1 as Relay Node 1
    participant Hop2 as Relay Node 2
    participant Hop3 as Relay Node 3
    participant Dest as Destination Node

    Client->>Router: Request anonymous route
    Router->>Router: Select diverse peer path
    Router->>Router: Generate ML-KEM keys for each hop

    Router->>Hop1: Establish shared secret
    Hop1->>Router: ML-KEM encapsulation
    Router->>Hop2: Establish shared secret
    Hop2->>Router: ML-KEM encapsulation
    Router->>Hop3: Establish shared secret
    Hop3->>Router: ML-KEM encapsulation
    Router-->>Client: Circuit established

    Client->>Router: Send message through circuit
    Router->>Router: Encrypt Layer 3 (for Hop3->Dest)
    Router->>Router: Encrypt Layer 2 (for Hop2->Hop3)
    Router->>Router: Encrypt Layer 1 (for Hop1->Hop2)

    Router->>Hop1: Send encrypted onion
    Hop1->>Hop1: Decrypt outer layer
    Hop1->>Hop2: Forward to next hop
    Hop2->>Hop2: Decrypt outer layer
    Hop2->>Hop3: Forward to next hop
    Hop3->>Hop3: Decrypt outer layer
    Hop3->>Dest: Deliver final message

    Dest-->>Client: Message received (via reverse path)
```

### Traffic Analysis Resistance

```mermaid
sequenceDiagram
    participant App as Application
    participant TAR as Traffic Analysis Resistance
    participant Mixer as Message Mixer
    participant Padder as Size Padder
    participant Timer as Timing Randomizer
    participant Network as Network Layer

    App->>TAR: Send message
    TAR->>Padder: Normalize message size
    Padder->>Padder: Add padding to standard size
    Padder-->>TAR: Padded message

    TAR->>Timer: Schedule transmission
    Timer->>Timer: Add random delay
    Timer-->>TAR: Delayed transmission

    TAR->>Mixer: Add to batch
    Mixer->>Mixer: Collect messages in batch
    
    alt Batch Full or Timeout
        Mixer->>Mixer: Shuffle batch order
        loop For each message in batch
            Mixer->>Network: Send message
            Network->>Network: Transmit via onion route
        end
    end

    par Cover Traffic Generation
        TAR->>TAR: Generate dummy message
        TAR->>Network: Send cover traffic
    end
```

## Dark Address Resolution

### .dark Domain Resolution Process

```mermaid
sequenceDiagram
    participant Client as Client
    participant Resolver as Dark Resolver
    participant Cache as Domain Cache
    participant DHT as Distributed Hash Table
    participant Validator as Quantum Fingerprint
    participant DNS as DNS Manager

    Client->>Resolver: Resolve "service.dark"
    Resolver->>Cache: Check cache
    
    alt Cache Hit
        Cache-->>Resolver: Cached record
        Resolver->>Validator: Verify fingerprint
        alt Fingerprint Valid
            Validator-->>Resolver: Valid
            Resolver-->>Client: Return address
        else Fingerprint Invalid
            Validator-->>Resolver: Invalid
            Resolver->>Cache: Evict entry
            Resolver->>DHT: Query network
        end
    else Cache Miss
        Resolver->>DHT: Query network
        DHT->>DHT: Lookup domain record
        DHT-->>Resolver: Domain record
        
        Resolver->>Validator: Verify quantum fingerprint
        Validator->>Validator: Verify ML-DSA signature
        
        alt Fingerprint Valid
            Validator-->>Resolver: Valid
            Resolver->>Cache: Cache record
            Resolver-->>Client: Return address
        else Fingerprint Invalid
            Validator-->>Resolver: Invalid
            Resolver-->>Client: Resolution failed
        end
    end
```

### Shadow Address Generation and Resolution

```mermaid
sequenceDiagram
    participant Service as Service
    participant Generator as Shadow Generator
    participant Resolver as Shadow Resolver
    participant Registry as Address Registry
    participant Client as Client

    Service->>Generator: Request shadow address
    Generator->>Generator: Generate random address
    Generator->>Generator: Set TTL (1 hour)
    Generator->>Registry: Bind service endpoint
    Registry-->>Generator: Binding confirmed
    Generator-->>Service: Shadow address created

    Note over Service,Client: Later: Client wants to connect

    Client->>Resolver: Resolve shadow address
    Resolver->>Registry: Lookup address
    
    alt Address Valid and Not Expired
        Registry-->>Resolver: Service endpoint
        Resolver-->>Client: Connection details
        Client->>Service: Connect to service
    else Address Expired
        Registry-->>Resolver: Address expired
        Resolver-->>Client: Resolution failed
    else Address Not Found
        Registry-->>Resolver: Not found
        Resolver-->>Client: Resolution failed
    end

    Note over Generator,Registry: Background cleanup
    Generator->>Registry: Clean expired addresses
    Registry->>Registry: Remove expired entries
```

## Peer Discovery

### Bootstrap and Peer Discovery Process

```mermaid
sequenceDiagram
    participant Node as New Node
    participant Bootstrap as Bootstrap Node
    participant DHT as Kademlia DHT
    participant Peer1 as Discovered Peer 1
    participant Peer2 as Discovered Peer 2
    participant Auth as Authentication

    Node->>Bootstrap: Connect to bootstrap node
    Bootstrap->>Auth: Verify node identity
    Auth->>Auth: Verify ML-DSA signature
    Auth-->>Bootstrap: Identity verified
    Bootstrap-->>Node: Connection accepted

    Node->>DHT: Join DHT network
    DHT->>DHT: Add node to routing table
    DHT-->>Node: DHT joined

    Node->>DHT: Discover peers
    DHT->>DHT: Query k-buckets
    DHT-->>Node: Return peer list

    par Connect to Multiple Peers
        Node->>Peer1: Establish connection
        Peer1->>Auth: Verify node identity
        Auth-->>Peer1: Identity verified
        Peer1-->>Node: Connection established

        Node->>Peer2: Establish connection
        Peer2->>Auth: Verify node identity
        Auth-->>Peer2: Identity verified
        Peer2-->>Node: Connection established
    end

    Node->>Node: Maintain peer connections
    
    loop Periodic Discovery
        Node->>DHT: Refresh peer list
        DHT-->>Node: Updated peers
        Node->>Node: Connect to new peers
    end
```

### Peer Authentication and Handshake

```mermaid
sequenceDiagram
    participant NodeA as Node A
    participant NodeB as Node B
    participant CryptoA as Node A Crypto
    parameter CryptoB as Node B Crypto
    participant Transport as Secure Transport

    NodeA->>NodeB: Connection request
    NodeB->>NodeA: Challenge nonce
    
    NodeA->>CryptoA: Sign challenge
    CryptoA->>CryptoA: Sign with ML-DSA
    CryptoA-->>NodeA: Signature
    NodeA->>NodeB: Send identity + signature

    NodeB->>CryptoB: Verify signature
    CryptoB->>CryptoB: Verify ML-DSA signature
    CryptoB-->>NodeB: Verification result
    
    alt Signature Valid
        NodeB->>NodeA: Send challenge response
        NodeA->>CryptoA: Verify response
        CryptoA-->>NodeA: Verification result
        
        alt Response Valid
            NodeA->>Transport: Establish secure channel
            NodeB->>Transport: Establish secure channel
            Transport-->>NodeA: Channel established
            Transport-->>NodeB: Channel established
            
            NodeA<<->>NodeB: Secure communication
        else Response Invalid
            NodeA->>NodeA: Reject connection
            NodeB->>NodeB: Log security event
        end
    else Signature Invalid
        NodeB->>NodeB: Reject connection
        NodeB->>NodeB: Log security event
    end
```

## Error Handling

### Consensus Failure Recovery

```mermaid
sequenceDiagram
    participant Node as Local Node
    participant Consensus as Consensus Engine
    participant Recovery as Recovery Manager
    participant Network as Network Layer
    participant State as State Manager

    Node->>Consensus: Submit vertex
    Consensus->>Network: Query peers
    Network-->>Consensus: Network timeout
    
    Consensus->>Recovery: Handle consensus failure
    Recovery->>Recovery: Analyze failure type
    
    alt Network Partition
        Recovery->>State: Enter partition mode
        Recovery->>Network: Attempt reconnection
        Network-->>Recovery: Reconnection status
        
        alt Reconnection Successful
            Recovery->>Consensus: Retry consensus
            Recovery->>State: Exit partition mode
        else Reconnection Failed
            Recovery->>State: Maintain partition mode
            Recovery->>Node: Consensus temporarily unavailable
        end
    else Peer Failures
        Recovery->>Network: Discover new peers
        Network-->>Recovery: New peer list
        Recovery->>Consensus: Retry with new peers
    else Byzantine Behavior
        Recovery->>Recovery: Identify malicious peers
        Recovery->>Network: Blacklist bad peers
        Recovery->>Consensus: Retry consensus
    end
```

### Network Failure and Recovery

```mermaid
sequenceDiagram
    participant App as Application
    participant Network as Network Manager
    participant Circuit as Circuit Manager
    participant Peer as Peer Manager
    participant Monitor as Health Monitor

    App->>Network: Send message
    Network->>Circuit: Use existing circuit
    Circuit->>Circuit: Attempt message transmission
    Circuit-->>Network: Circuit failure

    Network->>Monitor: Report circuit failure
    Monitor->>Monitor: Analyze failure pattern
    
    alt Single Hop Failure
        Monitor->>Circuit: Rebuild circuit (skip failed hop)
        Circuit->>Circuit: Create new route
        Circuit-->>Network: New circuit ready
        Network->>App: Retry message
    else Multiple Hop Failures
        Monitor->>Peer: Refresh peer list
        Peer->>Peer: Discover new peers
        Peer-->>Monitor: New peers available
        Monitor->>Circuit: Build new circuits
        Circuit-->>Network: New circuits ready
    else Network Partition
        Monitor->>Network: Enter degraded mode
        Network->>Network: Use direct connections only
        Network->>App: Limited functionality mode
        
        loop Recovery Attempts
            Monitor->>Peer: Attempt peer discovery
            Peer-->>Monitor: Discovery status
            alt Peers Found
                Monitor->>Network: Exit degraded mode
                Monitor->>Circuit: Rebuild anonymous circuits
            end
        end
    end
```

### Cryptographic Error Handling

```mermaid
sequenceDiagram
    participant App as Application
    participant Crypto as Crypto Manager
    participant KeyMgr as Key Manager
    participant SecMem as Secure Memory
    participant Logger as Security Logger

    App->>Crypto: Cryptographic operation
    Crypto->>Crypto: Perform operation
    
    alt Operation Successful
        Crypto-->>App: Result
    else Key Corruption Detected
        Crypto->>Logger: Log security event
        Crypto->>KeyMgr: Regenerate keys
        KeyMgr->>SecMem: Allocate new secure memory
        KeyMgr->>KeyMgr: Generate new keypair
        KeyMgr->>SecMem: Zeroize old keys
        KeyMgr-->>Crypto: New keys ready
        Crypto->>App: Retry operation
    else Memory Error
        Crypto->>Logger: Log critical error
        Crypto->>SecMem: Emergency memory cleanup
        SecMem->>SecMem: Zeroize all sensitive data
        Crypto->>App: Critical error - restart required
    else Timing Attack Detected
        Crypto->>Logger: Log potential attack
        Crypto->>Crypto: Add additional timing noise
        Crypto->>App: Operation completed (with countermeasures)
    end
```

## Protocol State Transitions

### Node Lifecycle State Machine

```mermaid
stateDiagram-v2
    [*] --> Initializing
    Initializing --> Initialized : All components loaded
    Initialized --> Connecting : Start network discovery
    Connecting --> Connected : Peers discovered and connected
    Connected --> Syncing : Begin state synchronization
    Syncing --> Operational : State synchronized
    Operational --> Degraded : Network issues detected
    Degraded --> Operational : Network recovered
    Operational --> Shutdown : Stop signal received
    Degraded --> Shutdown : Stop signal received
    Shutdown --> [*]
    
    state Operational {
        [*] --> Processing
        Processing --> Consensus : Message received
        Consensus --> Processing : Consensus complete
        Processing --> Routing : Outbound message
        Routing --> Processing : Message routed
    }
```

These sequence diagrams provide detailed insights into the QuDAG protocol's operation, showing how components interact during various operations while maintaining security and performance characteristics.