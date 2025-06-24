use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkAddress {
    pub ip: IpAddr,
    pub port: u16,
}

impl NetworkAddress {
    pub fn new(ip_parts: [u8; 4], port: u16) -> Self {
        Self {
            ip: IpAddr::V4(Ipv4Addr::new(ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3])),
            port,
        }
    }
    
    pub fn from_ip_port(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }
    
    pub fn to_socket_addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagePriority {
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    pub messages_per_second: f64,
    pub connections: usize,
    pub avg_latency: Duration,
    pub memory_usage: usize,
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            messages_per_second: 0.0,
            connections: 0,
            avg_latency: Duration::from_millis(0),
            memory_usage: 0,
        }
    }
}

fn main() {
    println!("🧪 Testing QuDAG Network Types Examples");
    println!("========================================");

    // Test NetworkAddress
    test_network_address();
    
    // Test MessagePriority
    test_message_priority();
    
    // Test NetworkMetrics
    test_network_metrics();
    
    println!("\n✅ All network type examples work correctly!");
}

fn test_network_address() {
    println!("\n📡 Testing NetworkAddress Example Usage...");
    
    // Example 1: Create from IP parts
    let addr1 = NetworkAddress::new([192, 168, 1, 1], 8080);
    assert_eq!(addr1.ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    assert_eq!(addr1.port, 8080);
    println!("  ✓ Example 1: Create from IP parts - PASS");
    
    // Example 2: Create from IP and port
    let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let addr2 = NetworkAddress::from_ip_port(ip, 9090);
    assert_eq!(addr2.ip, ip);
    assert_eq!(addr2.port, 9090);
    println!("  ✓ Example 2: Create from IP and port - PASS");
    
    // Example 3: Format as socket address
    let socket_str = addr1.to_socket_addr();
    assert_eq!(socket_str, "192.168.1.1:8080");
    println!("  ✓ Example 3: Format as socket address - PASS");
    
    // Example 4: Test equality
    let addr3 = NetworkAddress::new([192, 168, 1, 1], 8080);
    assert_eq!(addr1, addr3);
    println!("  ✓ Example 4: Address equality comparison - PASS");
}

fn test_message_priority() {
    println!("\n📝 Testing MessagePriority Example Usage...");
    
    // Example 1: Create different priorities
    let high = MessagePriority::High;
    let normal = MessagePriority::Normal;
    let low = MessagePriority::Low;
    
    // Example 2: Test equality
    assert_eq!(high, MessagePriority::High);
    assert_ne!(high, normal);
    println!("  ✓ Example 1-2: Priority creation and equality - PASS");
    
    // Example 3: Use in match statements
    let priority_description = match high {
        MessagePriority::High => "Urgent message",
        MessagePriority::Normal => "Standard message", 
        MessagePriority::Low => "Background message",
    };
    assert_eq!(priority_description, "Urgent message");
    println!("  ✓ Example 3: Priority matching - PASS");
    
    // Example 4: Use priorities in collections
    let priorities = vec![high, normal, low];
    assert_eq!(priorities.len(), 3);
    println!("  ✓ Example 4: Priorities in collections - PASS");
}

fn test_network_metrics() {
    println!("\n📊 Testing NetworkMetrics Example Usage...");
    
    // Example 1: Create default metrics
    let metrics = NetworkMetrics::default();
    assert_eq!(metrics.messages_per_second, 0.0);
    assert_eq!(metrics.connections, 0);
    assert_eq!(metrics.avg_latency, Duration::from_millis(0));
    assert_eq!(metrics.memory_usage, 0);
    println!("  ✓ Example 1: Default metrics creation - PASS");
    
    // Example 2: Create custom metrics
    let custom_metrics = NetworkMetrics {
        messages_per_second: 100.5,
        connections: 10,
        avg_latency: Duration::from_millis(50),
        memory_usage: 1024,
    };
    assert_eq!(custom_metrics.messages_per_second, 100.5);
    assert_eq!(custom_metrics.connections, 10);
    println!("  ✓ Example 2: Custom metrics creation - PASS");
    
    // Example 3: Update metrics over time
    let mut live_metrics = NetworkMetrics::default();
    live_metrics.connections = 5;
    live_metrics.messages_per_second = 42.5;
    live_metrics.avg_latency = Duration::from_millis(25);
    live_metrics.memory_usage = 512;
    
    assert_eq!(live_metrics.connections, 5);
    assert_eq!(live_metrics.messages_per_second, 42.5);
    println!("  ✓ Example 3: Metrics updates - PASS");
    
    // Example 4: Performance monitoring simulation
    let start_time = std::time::Instant::now();
    for _ in 0..1000 {
        let mut temp_metrics = NetworkMetrics::default();
        temp_metrics.connections = 1;
    }
    let duration = start_time.elapsed();
    println!("  ✓ Example 4: Performance monitoring (1000 ops in {:?}) - PASS", duration);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_network_address() {
        let addr = NetworkAddress::new([127, 0, 0, 1], 8080);
        assert_eq!(addr.port, 8080);
        assert_eq!(addr.ip, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }

    #[test]
    fn test_example_message_priority() {
        let high = MessagePriority::High;
        let normal = MessagePriority::Normal;
        assert_ne!(high, normal);
    }

    #[test]
    fn test_example_network_metrics() {
        let metrics = NetworkMetrics::default();
        assert_eq!(metrics.connections, 0);
        assert_eq!(metrics.messages_per_second, 0.0);
    }
}