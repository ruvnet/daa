//! Network behavior composition for P2P communication

use libp2p::{
    swarm::NetworkBehaviour, PeerId,
    kad::{Behaviour as Kademlia, Event as KademliaEvent, store::MemoryStore},
    gossipsub::{Behaviour as Gossipsub, Event as GossipsubEvent, MessageAuthenticity, ValidationMode, IdentTopic},
    identify::{Behaviour as Identify, Config as IdentifyConfig, Event as IdentifyEvent},
    ping::{Behaviour as Ping, Event as PingEvent},
};
use std::time::Duration;
use anyhow::{Result, anyhow};

use super::SwarmConfig;

/// Composed network behavior
#[derive(NetworkBehaviour)]
pub struct NetworkBehavior {
    pub kademlia: Kademlia<MemoryStore>,
    pub gossipsub: Gossipsub,
    pub identify: Identify,
    pub ping: Ping,
}

impl NetworkBehavior {
    pub fn new(
        local_key: libp2p::identity::Keypair,
        config: &SwarmConfig,
    ) -> Result<Self> {
        let local_peer_id = PeerId::from(local_key.public());

        // Kademlia
        let store = MemoryStore::new(local_peer_id);
        let mut kademlia_config = config.kademlia_config.clone();
        kademlia_config.set_query_timeout(Duration::from_secs(60));
        let kademlia = Kademlia::with_config(local_peer_id, store, kademlia_config);

        // Gossipsub
        let gossipsub = {
            let message_authenticity = MessageAuthenticity::Signed(local_key.clone());
            let mut gossipsub_config = config.gossipsub_config.clone();
            // Note: In libp2p 0.53, validation_mode and message_id are set differently
            // For now we'll use the default config and customize after construction
            
            let mut gossipsub = Gossipsub::new(message_authenticity, gossipsub_config)
                .map_err(|e| anyhow::anyhow!("Failed to create gossipsub: {}", e))?;
            
            // Subscribe to gradient topic
            let topic = IdentTopic::new("gradients");
            gossipsub.subscribe(&topic)
                .map_err(|e| anyhow!("Failed to subscribe to topic: {}", e))?;
            
            gossipsub
        };

        // Identify
        let identify = Identify::new(IdentifyConfig::new(
            "/daa-compute/1.0.0".to_string(),
            local_key.public(),
        ));

        // Ping
        let ping = Ping::default();

        Ok(Self {
            kademlia,
            gossipsub,
            identify,
            ping,
        })
    }
}

/// Composed event type
#[derive(Debug)]
pub enum ComposedEvent {
    Kademlia(KademliaEvent),
    Gossipsub(GossipsubEvent),
    Identify(IdentifyEvent),
    Ping(PingEvent),
}

impl From<KademliaEvent> for ComposedEvent {
    fn from(event: KademliaEvent) -> Self {
        Self::Kademlia(event)
    }
}

impl From<GossipsubEvent> for ComposedEvent {
    fn from(event: GossipsubEvent) -> Self {
        Self::Gossipsub(event)
    }
}

impl From<IdentifyEvent> for ComposedEvent {
    fn from(event: IdentifyEvent) -> Self {
        Self::Identify(event)
    }
}

impl From<PingEvent> for ComposedEvent {
    fn from(event: PingEvent) -> Self {
        Self::Ping(event)
    }
}