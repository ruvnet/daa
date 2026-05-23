//! Configuration command (library stubs — runtime logic lives in main.rs)
//!
//! This module is compiled in both library and binary contexts. The binary's
//! full config handler is inlined in main.rs to avoid cross-context type
//! dependencies.

// No public API exposed here; the lib uses commands::config for documentation
// purposes. The binary dispatches config commands via handle_config_command in
// main.rs.
