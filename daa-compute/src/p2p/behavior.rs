//! Network behavior composition for P2P communication

use libp2p::{
    NetworkBehaviour, PeerId,
    kad::{Kademlia, KademliaConfig, KademliaEvent, store::MemoryStore},
    gossipsub::{Gossipsub, GossipsubEvent, MessageAuthenticity, ValidationMode, Topic},
    identify::{Identify, IdentifyConfig, IdentifyEvent},
    ping::{Ping, PingEvent},
    mdns::{Mdns, MdnsEvent},
    relay::{self, Relay},
    autonat::{self, AutoNat},
    upnp::tokio::Behaviour as Upnp,
    dcutr,
};
use std::time::Duration;
use anyhow::Result;

use super::SwarmConfig;

/// Composed network behavior
#[derive(NetworkBehaviour)]
#[behaviour(event_process = false)]
pub struct NetworkBehavior {
    pub kademlia: Kademlia<MemoryStore>,
    pub gossipsub: Gossipsub,
    pub identify: Identify,
    pub ping: Ping,
    pub mdns: Option<Mdns>,
    pub relay: Option<Relay>,
    pub autonat: Option<AutoNat>,
    pub upnp: Option<Upnp>,
    pub dcutr: Option<dcutr::Behaviour>,
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
            gossipsub_config.validation_mode = ValidationMode::Strict;
            gossipsub_config.message_id_fn = |message| {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                message.data.hash(&mut hasher);
                hasher.finish().to_string()
            };
            
            let mut gossipsub = Gossipsub::new(message_authenticity, gossipsub_config)?;
            
            // Subscribe to gradient topic
            gossipsub.subscribe(&crate::p2p::gradient::GRADIENT_TOPIC)?;
            
            gossipsub
        };

        // Identify
        let identify = Identify::new(IdentifyConfig::new(
            "/daa-compute/1.0.0".to_string(),
            local_key.public(),
        ));

        // Ping
        let ping = Ping::default();

        // mDNS
        let mdns = if config.enable_mdns {
            Some(Mdns::new(Default::default())?)
        } else {
            None
        };

        // Relay
        let relay = if config.enable_relay {
            Some(Relay::new(local_peer_id, Default::default()))
        } else {
            None
        };

        // AutoNAT
        let autonat = if config.enable_nat_traversal {
            Some(AutoNat::new(local_peer_id, Default::default()))
        } else {
            None
        };

        // UPnP
        let upnp = if config.enable_nat_traversal {
            Some(Upnp::default())
        } else {
            None
        };

        // DCUTR (Direct Connection Upgrade through Relay)
        let dcutr = if config.enable_relay && config.enable_nat_traversal {
            Some(dcutr::Behaviour::new(local_peer_id))
        } else {
            None
        };

        Ok(Self {
            kademlia,
            gossipsub,
            identify,
            ping,
            mdns,
            relay,
            autonat,
            upnp,
            dcutr,
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
    Mdns(MdnsEvent),
    Relay(relay::Event),
    AutoNat(autonat::Event),
    Upnp(upnp::Event),
    Dcutr(dcutr::Event),
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

impl From<MdnsEvent> for ComposedEvent {
    fn from(event: MdnsEvent) -> Self {
        Self::Mdns(event)
    }
}

impl From<relay::Event> for ComposedEvent {
    fn from(event: relay::Event) -> Self {
        Self::Relay(event)
    }
}

impl From<autonat::Event> for ComposedEvent {
    fn from(event: autonat::Event) -> Self {
        Self::AutoNat(event)
    }
}

impl From<upnp::Event> for ComposedEvent {
    fn from(event: upnp::Event) -> Self {
        Self::Upnp(event)
    }
}

impl From<dcutr::Event> for ComposedEvent {
    fn from(event: dcutr::Event) -> Self {
        Self::Dcutr(event)
    }
}