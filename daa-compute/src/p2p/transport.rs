//! Transport layer configuration for P2P network
//!
//! Supports TCP, WebSocket, and WebRTC transports with automatic
//! multiplexing and encryption.

use anyhow::Result;
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::Boxed, upgrade::Version},
    dns, noise, tcp, websocket, yamux, PeerId, Transport,
};
use std::time::Duration;

#[cfg(all(target_arch = "wasm32", feature = "browser"))]
use libp2p_webrtc_websys as webrtc;

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub enable_tcp: bool,
    pub enable_websocket: bool,
    /// WebRTC transport; only honoured by browser (wasm32) builds via `create_wasm_transport`.
    pub enable_webrtc: bool,
    pub enable_relay: bool,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            enable_tcp: true,
            enable_websocket: true,
            enable_webrtc: false,
            enable_relay: true,
        }
    }
}

/// Create a transport with the given configuration
pub fn create_transport(
    local_key: &libp2p::identity::Keypair,
    config: TransportConfig,
) -> Result<Boxed<(PeerId, StreamMuxerBox)>> {
    let noise_config = noise::Config::new(local_key).unwrap();
    let yamux_config = yamux::Config::default();

    let tcp_transport = {
        let tcp = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true));
        let dns_tcp = dns::tokio::Transport::system(tcp).unwrap();
        dns_tcp
            .upgrade(Version::V1)
            .authenticate(noise_config.clone())
            .multiplex(yamux_config.clone())
            .timeout(Duration::from_secs(20))
            .boxed()
    };

    // Combine transports
    let transport = if config.enable_websocket {
        let ws_dns_tcp = {
            let tcp = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true));
            let dns_tcp = dns::tokio::Transport::system(tcp).unwrap();
            websocket::Config::new(dns_tcp)
        };

        let ws_transport = ws_dns_tcp
            .upgrade(Version::V1)
            .authenticate(noise_config.clone())
            .multiplex(yamux_config.clone())
            .timeout(Duration::from_secs(20))
            .boxed();

        tcp_transport.or_transport(ws_transport).boxed()
    } else {
        use libp2p::core::transport::dummy::DummyTransport;
        tcp_transport.or_transport(DummyTransport::new()).boxed()
    };

    // WebRTC is only available to browser (wasm32) nodes, see `create_wasm_transport`.
    // Native nodes reach browser peers over WebSocket.

    // Add relay transport
    // TODO: Fix relay client API for libp2p 0.53 - Transport::new is now private
    // Need to use SwarmBuilder integration instead
    if config.enable_relay {
        // Relay transport integration moved to SwarmBuilder in libp2p 0.53
        // This will be handled in the main swarm construction
    }

    Ok(transport
        .map(|either_output, _| match either_output {
            futures::future::Either::Left((peer_id, muxer)) => (peer_id, muxer),
            futures::future::Either::Right((peer_id, muxer)) => (peer_id, muxer),
        })
        .boxed())
}

/// Create a WASM-compatible transport for browser nodes
#[cfg(target_arch = "wasm32")]
pub fn create_wasm_transport(
    local_key: &libp2p::identity::Keypair,
) -> Result<Boxed<(PeerId, StreamMuxerBox)>> {
    use wasm_bindgen_futures::spawn_local;

    let noise_config = noise::Config::new(local_key).unwrap();
    let yamux_config = yamux::Config::default();

    // WebSocket transport for WASM
    let ws_transport = websocket::WsConfig::new_plain_text(websocket::tokio::Transport::default())
        .upgrade(Version::V1)
        .authenticate(noise_config.clone())
        .multiplex(yamux_config.clone())
        .boxed();

    // WebRTC transport for direct browser-to-browser
    #[cfg(feature = "browser")]
    {
        let webrtc_transport = webrtc::Transport::new(webrtc::Config::new(local_key));

        Ok(ws_transport
            .or_transport(webrtc_transport)
            .map(|output, _| output)
            .boxed())
    }

    #[cfg(not(feature = "browser"))]
    Ok(ws_transport)
}

/// STUN/TURN configuration for NAT traversal
#[derive(Debug, Clone)]
pub struct IceConfig {
    pub stun_servers: Vec<String>,
    pub turn_servers: Vec<TurnServer>,
}

#[derive(Debug, Clone)]
pub struct TurnServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

impl Default for IceConfig {
    fn default() -> Self {
        Self {
            stun_servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
                "stun:stun2.l.google.com:19302".to_string(),
                "stun:stun3.l.google.com:19302".to_string(),
                "stun:stun4.l.google.com:19302".to_string(),
            ],
            turn_servers: vec![],
        }
    }
}
