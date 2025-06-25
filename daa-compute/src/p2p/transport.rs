//! Transport layer configuration for P2P network
//!
//! Supports TCP, WebSocket, and WebRTC transports with automatic
//! multiplexing and encryption.

use libp2p::{
    Transport, PeerId,
    core::{transport::Boxed, muxing::StreamMuxerBox},
    tcp, websocket, noise, yamux,
    dns,
};
use std::time::Duration;
use anyhow::Result;

#[cfg(feature = "browser")]
use libp2p_webrtc as webrtc;

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub enable_tcp: bool,
    pub enable_websocket: bool,
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
    let noise_config = noise::NoiseAuthenticated::xx(local_key)?;
    let yamux_config = yamux::YamuxConfig::default();

    let mut transport = {
        let tcp = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true));
        let dns_tcp = dns::DnsConfig::system(tcp)?;
        dns_tcp
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise_config.clone())
            .multiplex(yamux_config.clone())
            .timeout(Duration::from_secs(20))
            .boxed()
    };

    // Add WebSocket support
    if config.enable_websocket {
        let ws_dns_tcp = {
            let tcp = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true));
            let dns_tcp = dns::DnsConfig::system(tcp)?;
            websocket::WsConfig::new(dns_tcp)
        };
        
        let ws_transport = ws_dns_tcp
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise_config.clone())
            .multiplex(yamux_config.clone())
            .timeout(Duration::from_secs(20))
            .boxed();
            
        transport = transport.or_transport(ws_transport).boxed();
    }

    // Add WebRTC support for browser compatibility
    #[cfg(feature = "browser")]
    if config.enable_webrtc {
        let webrtc_config = webrtc::Config::new(local_key)?;
        let webrtc_transport = webrtc::tokio::Transport::new(webrtc_config);
        transport = transport.or_transport(webrtc_transport).boxed();
    }

    // Add relay transport
    if config.enable_relay {
        transport = libp2p::relay::client::new(transport);
    }

    Ok(transport)
}

/// Create a WASM-compatible transport for browser nodes
#[cfg(target_arch = "wasm32")]
pub fn create_wasm_transport(
    local_key: &libp2p::identity::Keypair,
) -> Result<Boxed<(PeerId, StreamMuxerBox)>> {
    use wasm_bindgen_futures::spawn_local;
    
    let noise_config = noise::NoiseAuthenticated::xx(local_key)?;
    let yamux_config = yamux::YamuxConfig::default();

    // WebSocket transport for WASM
    let ws_transport = websocket::WsConfig::new(websocket::framed::WsConfig::new())
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(noise_config.clone())
        .multiplex(yamux_config.clone())
        .boxed();

    // WebRTC transport for direct browser-to-browser
    #[cfg(feature = "browser")]
    {
        let webrtc_config = webrtc::Config::new(local_key)?;
        let webrtc_transport = webrtc::Transport::new(webrtc_config);
        
        ws_transport
            .or_transport(webrtc_transport)
            .map(|output, _| output)
            .boxed()
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

/// Configure ICE servers for WebRTC
#[cfg(feature = "browser")]
pub fn configure_ice_servers(ice_config: &IceConfig) -> webrtc::IceServers {
    let mut servers = webrtc::IceServers::new();
    
    // Add STUN servers
    for stun in &ice_config.stun_servers {
        servers.add_stun(stun);
    }
    
    // Add TURN servers
    for turn in &ice_config.turn_servers {
        for url in &turn.urls {
            if let (Some(username), Some(credential)) = (&turn.username, &turn.credential) {
                servers.add_turn(url, username, credential);
            }
        }
    }
    
    servers
}