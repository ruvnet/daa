use proptest::prelude::*;
use qudag_cli::cli::Args;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

fn is_valid_socket_addr(s: &str) -> bool {
    SocketAddr::from_str(s).is_ok()
}

fn is_valid_toml_file(path: &str) -> bool {
    path.ends_with(".toml") && !path.contains("..") && !path.contains('/')
}

fn is_valid_log_level(level: &str) -> bool {
    matches!(level, "error" | "warn" | "info" | "debug" | "trace")
}

proptest! {
    #[test]
    fn test_valid_bind_address(ip in r"[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}", port in 1024u16..65535u16) {
        let addr = format!("{}:{}", ip, port);
        let args = Args::try_parse_from(&[
            "qudag",
            "node",
            "start",
            "--config",
            "config.toml",
            "--bind",
            &addr
        ]);

        // Test will fail until Args struct is implemented
        prop_assert!(args.is_ok());
        let args = args.unwrap();
        prop_assert!(is_valid_socket_addr(&args.bind.unwrap()));
    }

    #[test]
    fn test_valid_peer_addresses(
        peers in prop::collection::vec(
            r"[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}:[0-9]{4,5}",
            1..10
        )
    ) {
        let peer_list = peers.join(",");
        let args = Args::try_parse_from(&[
            "qudag",
            "node",
            "start",
            "--config",
            "config.toml",
            "--peers",
            &peer_list
        ]);

        // Test will fail until Args struct is implemented
        prop_assert!(args.is_ok());
        let args = args.unwrap();
        for peer in args.peers.unwrap() {
            prop_assert!(is_valid_socket_addr(&peer));
        }
    }

    #[test]
    fn test_valid_config_paths(path in r"[a-zA-Z0-9_-]+\.toml") {
        let args = Args::try_parse_from(&[
            "qudag",
            "node",
            "start",
            "--config",
            &path
        ]);

        // Test will fail until Args struct is implemented
        prop_assert!(args.is_ok());
        let args = args.unwrap();
        prop_assert!(is_valid_toml_file(&args.config.unwrap()));
    }

    #[test]
    fn test_valid_log_levels(level in prop::sample::select(&["error", "warn", "info", "debug", "trace"])) {
        let args = Args::try_parse_from(&[
            "qudag",
            "node",
            "start",
            "--config",
            "config.toml",
            "--log-level",
            &level
        ]);

        // Test will fail until Args struct is implemented
        prop_assert!(args.is_ok());
        let args = args.unwrap();
        prop_assert!(is_valid_log_level(&args.log_level.unwrap()));
    }
}

#[test]
fn test_multiple_valid_combinations() {
    proptest!(|(
        bind_addr in r"[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}:[0-9]{4,5}",
        config in r"[a-zA-Z0-9_-]+\.toml",
        level in prop::sample::select(&["error", "warn", "info", "debug", "trace"])
    )| {
        let args = Args::try_parse_from(&[
            "qudag",
            "node",
            "start",
            "--config",
            &config,
            "--bind",
            &bind_addr,
            "--log-level",
            &level
        ]);

        // Test will fail until Args struct is implemented
        prop_assert!(args.is_ok());
        let args = args.unwrap();
        prop_assert!(is_valid_socket_addr(&args.bind.unwrap()));
        prop_assert!(is_valid_toml_file(&args.config.unwrap()));
        prop_assert!(is_valid_log_level(&args.log_level.unwrap()));
    });
}

#[test]
fn test_invalid_combinations() {
    proptest!(|(
        bind_addr in r"[a-zA-Z]+:[0-9]+",  // Invalid IP format
        config in r"[a-zA-Z0-9_-]+\.txt",  // Invalid file extension
        level in r"[A-Z]+"  // Invalid log level
    )| {
        let args = Args::try_parse_from(&[
            "qudag",
            "node",
            "start",
            "--config",
            &config,
            "--bind",
            &bind_addr,
            "--log-level",
            &level
        ]);

        // Test will fail until Args struct is implemented with proper validation
        prop_assert!(args.is_err());
    });
}