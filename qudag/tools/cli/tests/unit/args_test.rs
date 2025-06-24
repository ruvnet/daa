#[cfg(test)]
mod cli_args_tests {
    use clap::Parser;
    use std::path::PathBuf;

    // Define the CLI structures matching the actual implementation
    #[derive(Parser, Debug, PartialEq)]
    #[command(name = "qudag")]
    #[command(about = "QuDAG Protocol CLI", long_about = None)]
    struct TestCli {
        #[command(subcommand)]
        command: TestCommands,
    }

    #[derive(clap::Subcommand, Debug, PartialEq)]
    enum TestCommands {
        Start {
            #[arg(short, long, default_value = "8000")]
            port: u16,
            #[arg(short, long)]
            data_dir: Option<PathBuf>,
            #[arg(short, long, default_value = "info")]
            log_level: String,
        },
        Stop,
        Status,
        Peer {
            #[command(subcommand)]
            command: TestPeerCommands,
        },
        Network {
            #[command(subcommand)]
            command: TestNetworkCommands,
        },
        Address {
            #[command(subcommand)]
            command: TestAddressCommands,
        },
    }

    #[derive(clap::Subcommand, Debug, PartialEq)]
    enum TestPeerCommands {
        List,
        Add { address: String },
        Remove { address: String },
    }

    #[derive(clap::Subcommand, Debug, PartialEq)]
    enum TestNetworkCommands {
        Stats,
        Test,
    }

    #[derive(clap::Subcommand, Debug, PartialEq)]
    enum TestAddressCommands {
        Register { domain: String },
        Resolve { domain: String },
        Shadow {
            #[arg(long, default_value = "3600")]
            ttl: u64,
        },
        Fingerprint {
            #[arg(long)]
            data: String,
        },
    }

    #[test]
    fn test_parse_start_command() {
        let cli = TestCli::try_parse_from(&["qudag", "start"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            TestCommands::Start { port, data_dir, log_level } => {
                assert_eq!(port, 8000);
                assert_eq!(data_dir, None);
                assert_eq!(log_level, "info");
            }
            _ => panic!("Expected Start command"),
        }
    }

    #[test]
    fn test_parse_start_with_options() {
        let cli = TestCli::try_parse_from(&[
            "qudag", "start", 
            "--port", "9000", 
            "--data-dir", "/tmp/data",
            "--log-level", "debug"
        ]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            TestCommands::Start { port, data_dir, log_level } => {
                assert_eq!(port, 9000);
                assert_eq!(data_dir, Some(PathBuf::from("/tmp/data")));
                assert_eq!(log_level, "debug");
            }
            _ => panic!("Expected Start command"),
        }
    }

    #[test]
    fn test_parse_stop_command() {
        let cli = TestCli::try_parse_from(&["qudag", "stop"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            TestCommands::Stop => {},
            _ => panic!("Expected Stop command"),
        }
    }

    #[test]
    fn test_parse_status_command() {
        let cli = TestCli::try_parse_from(&["qudag", "status"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            TestCommands::Status => {},
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn test_parse_peer_list_command() {
        let cli = TestCli::try_parse_from(&["qudag", "peer", "list"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            TestCommands::Peer { command } => {
                assert_eq!(command, TestPeerCommands::List);
            }
            _ => panic!("Expected Peer command"),
        }
    }

    #[test]
    fn test_parse_peer_add_command() {
        let cli = TestCli::try_parse_from(&["qudag", "peer", "add", "127.0.0.1:8000"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            TestCommands::Peer { command } => {
                match command {
                    TestPeerCommands::Add { address } => {
                        assert_eq!(address, "127.0.0.1:8000");
                    }
                    _ => panic!("Expected Add subcommand"),
                }
            }
            _ => panic!("Expected Peer command"),
        }
    }

    #[test]
    fn test_parse_peer_remove_command() {
        let cli = TestCli::try_parse_from(&["qudag", "peer", "remove", "127.0.0.1:8000"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            TestCommands::Peer { command } => {
                match command {
                    TestPeerCommands::Remove { address } => {
                        assert_eq!(address, "127.0.0.1:8000");
                    }
                    _ => panic!("Expected Remove subcommand"),
                }
            }
            _ => panic!("Expected Peer command"),
        }
    }

    #[test]
    fn test_parse_network_stats_command() {
        let cli = TestCli::try_parse_from(&["qudag", "network", "stats"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            TestCommands::Network { command } => {
                assert_eq!(command, TestNetworkCommands::Stats);
            }
            _ => panic!("Expected Network command"),
        }
    }

    #[test]
    fn test_parse_address_register_command() {
        let cli = TestCli::try_parse_from(&["qudag", "address", "register", "example.dark"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            TestCommands::Address { command } => {
                match command {
                    TestAddressCommands::Register { domain } => {
                        assert_eq!(domain, "example.dark");
                    }
                    _ => panic!("Expected Register subcommand"),
                }
            }
            _ => panic!("Expected Address command"),
        }
    }

    #[test]
    fn test_parse_invalid_command() {
        let cli = TestCli::try_parse_from(&["qudag", "invalid"]);
        assert!(cli.is_err());
    }

    #[test]
    fn test_parse_help_flag() {
        let cli = TestCli::try_parse_from(&["qudag", "--help"]);
        assert!(cli.is_err()); // Help flag exits with error code
    }

    #[test]
    fn test_parse_version_flag() {
        let cli = TestCli::try_parse_from(&["qudag", "--version"]);
        assert!(cli.is_err()); // Version flag exits with error code
    }

    #[test]
    fn test_parse_start_with_invalid_port() {
        let cli = TestCli::try_parse_from(&["qudag", "start", "--port", "invalid"]);
        assert!(cli.is_err());
    }

    #[test]
    fn test_parse_start_with_port_zero() {
        let cli = TestCli::try_parse_from(&["qudag", "start", "--port", "0"]);
        assert!(cli.is_ok()); // Port 0 is valid (system assigned)
    }

    #[test]
    fn test_parse_start_with_high_port() {
        let cli = TestCli::try_parse_from(&["qudag", "start", "--port", "65535"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_missing_required_argument() {
        let cli = TestCli::try_parse_from(&["qudag", "peer", "add"]);
        assert!(cli.is_err());
    }

    #[test]
    fn test_address_shadow_with_ttl() {
        let cli = TestCli::try_parse_from(&["qudag", "address", "shadow", "--ttl", "7200"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            TestCommands::Address { command } => {
                match command {
                    TestAddressCommands::Shadow { ttl } => {
                        assert_eq!(ttl, 7200);
                    }
                    _ => panic!("Expected Shadow subcommand"),
                }
            }
            _ => panic!("Expected Address command"),
        }
    }

    #[test]
    fn test_address_fingerprint_with_data() {
        let cli = TestCli::try_parse_from(&["qudag", "address", "fingerprint", "--data", "test-data"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            TestCommands::Address { command } => {
                match command {
                    TestAddressCommands::Fingerprint { data } => {
                        assert_eq!(data, "test-data");
                    }
                    _ => panic!("Expected Fingerprint subcommand"),
                }
            }
            _ => panic!("Expected Address command"),
        }
    }
}