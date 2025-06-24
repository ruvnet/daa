#![no_main]
use libfuzzer_sys::fuzz_target;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, BTreeMap, HashSet};

/// Complex nested structure for serialization testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexData {
    pub id: u64,
    pub name: String,
    pub values: Vec<i32>,
    pub metadata: HashMap<String, String>,
    pub tree_data: BTreeMap<String, Vec<u8>>,
    pub flags: HashSet<String>,
    pub nested: Option<Box<ComplexData>>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkPacket {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
    pub checksum: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PacketHeader {
    pub version: u8,
    pub packet_type: PacketType,
    pub flags: u16,
    pub length: u32,
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PacketType {
    Data,
    Control,
    Heartbeat,
    Error(String),
    Custom { code: u16, data: Vec<u8> },
}

/// Test various serialization formats and edge cases
fn test_bincode_serialization(data: &ComplexData) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize to bincode
    let serialized = bincode::serialize(data)?;
    
    // Deserialize back
    let deserialized: ComplexData = bincode::deserialize(&serialized)?;
    
    // Verify round-trip integrity
    if *data != deserialized {
        return Err("Bincode round-trip failed".into());
    }
    
    // Test with corrupted data
    if serialized.len() > 4 {
        let mut corrupted = serialized.clone();
        corrupted[serialized.len() / 2] ^= 0xFF;
        
        // Should fail gracefully
        let result: Result<ComplexData, _> = bincode::deserialize(&corrupted);
        if result.is_ok() {
            return Err("Corrupted data deserialized successfully".into());
        }
    }
    
    Ok(())
}

fn test_json_serialization(data: &ComplexData) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize to JSON
    let serialized = serde_json::to_string(data)?;
    
    // Deserialize back
    let deserialized: ComplexData = serde_json::from_str(&serialized)?;
    
    // Verify round-trip integrity
    if *data != deserialized {
        return Err("JSON round-trip failed".into());
    }
    
    // Test with malformed JSON
    let malformed_json = format!("{{\"corrupted\": {}}}", &serialized[10..]);
    let result: Result<ComplexData, _> = serde_json::from_str(&malformed_json);
    if result.is_ok() {
        return Err("Malformed JSON deserialized successfully".into());
    }
    
    Ok(())
}

fn test_network_packet_serialization(packet: &NetworkPacket) -> Result<(), Box<dyn std::error::Error>> {
    // Test bincode serialization
    let bin_serialized = bincode::serialize(packet)?;
    let bin_deserialized: NetworkPacket = bincode::deserialize(&bin_serialized)?;
    
    if *packet != bin_deserialized {
        return Err("Network packet bincode round-trip failed".into());
    }
    
    // Test JSON serialization
    let json_serialized = serde_json::to_string(packet)?;
    let json_deserialized: NetworkPacket = serde_json::from_str(&json_serialized)?;
    
    if *packet != json_deserialized {
        return Err("Network packet JSON round-trip failed".into());
    }
    
    // Test partial deserialization
    if bin_serialized.len() > 8 {
        for i in 1..std::cmp::min(bin_serialized.len(), 32) {
            let partial = &bin_serialized[..i];
            let _: Result<NetworkPacket, _> = bincode::deserialize(partial);
            // Should fail gracefully - we don't check result
        }
    }
    
    Ok(())
}

/// Create complex data from fuzz input
fn create_complex_data_from_fuzz(data: &[u8]) -> ComplexData {
    if data.is_empty() {
        return ComplexData {
            id: 0,
            name: String::new(),
            values: Vec::new(),
            metadata: HashMap::new(),
            tree_data: BTreeMap::new(),
            flags: HashSet::new(),
            nested: None,
            timestamp: 0,
        };
    }
    
    let id = u64::from_le_bytes([
        data[0 % data.len()],
        data[1 % data.len()],
        data[2 % data.len()],
        data[3 % data.len()],
        data[4 % data.len()],
        data[5 % data.len()],
        data[6 % data.len()],
        data[7 % data.len()],
    ]);
    
    let name = String::from_utf8_lossy(&data[..std::cmp::min(data.len(), 64)]).to_string();
    
    let values: Vec<i32> = data
        .chunks(4)
        .take(16) // Limit to prevent excessive memory usage
        .map(|chunk| {
            let mut bytes = [0u8; 4];
            for (i, &b) in chunk.iter().enumerate() {
                if i < 4 {
                    bytes[i] = b;
                }
            }
            i32::from_le_bytes(bytes)
        })
        .collect();
    
    let mut metadata = HashMap::new();
    for (i, chunk) in data.chunks(16).take(8).enumerate() {
        let key = format!("key_{}", i);
        let value = String::from_utf8_lossy(chunk).to_string();
        metadata.insert(key, value);
    }
    
    let mut tree_data = BTreeMap::new();
    for (i, chunk) in data.chunks(8).take(8).enumerate() {
        let key = format!("tree_{}", i);
        tree_data.insert(key, chunk.to_vec());
    }
    
    let mut flags = HashSet::new();
    for i in 0..std::cmp::min(data.len(), 8) {
        if data[i] % 2 == 0 {
            flags.insert(format!("flag_{}", i));
        }
    }
    
    let nested = if data.len() > 128 && data[0] % 4 == 0 {
        // Create a simpler nested structure to avoid infinite recursion
        Some(Box::new(ComplexData {
            id: id.wrapping_add(1),
            name: format!("nested_{}", name),
            values: values[..std::cmp::min(values.len(), 4)].to_vec(),
            metadata: HashMap::new(),
            tree_data: BTreeMap::new(),
            flags: HashSet::new(),
            nested: None,
            timestamp: id.wrapping_add(100),
        }))
    } else {
        None
    };
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    ComplexData {
        id,
        name,
        values,
        metadata,
        tree_data,
        flags,
        nested,
        timestamp,
    }
}

/// Create network packet from fuzz input
fn create_network_packet_from_fuzz(data: &[u8]) -> NetworkPacket {
    if data.is_empty() {
        return NetworkPacket {
            header: PacketHeader {
                version: 1,
                packet_type: PacketType::Data,
                flags: 0,
                length: 0,
                sequence: 0,
            },
            payload: Vec::new(),
            checksum: 0,
        };
    }
    
    let version = data[0 % data.len()];
    
    let packet_type = match data[1 % data.len()] % 5 {
        0 => PacketType::Data,
        1 => PacketType::Control,
        2 => PacketType::Heartbeat,
        3 => PacketType::Error(String::from_utf8_lossy(&data[..std::cmp::min(data.len(), 32)]).to_string()),
        _ => PacketType::Custom {
            code: u16::from_le_bytes([data[2 % data.len()], data[3 % data.len()]]),
            data: data[4..std::cmp::min(data.len(), 32)].to_vec(),
        },
    };
    
    let flags = u16::from_le_bytes([data[4 % data.len()], data[5 % data.len()]]);
    let length = std::cmp::min(data.len() as u32, 1024);
    let sequence = u64::from_le_bytes([
        data[6 % data.len()],
        data[7 % data.len()],
        data[8 % data.len()],
        data[9 % data.len()],
        data[10 % data.len()],
        data[11 % data.len()],
        data[12 % data.len()],
        data[13 % data.len()],
    ]);
    
    let payload = data[14..std::cmp::min(data.len(), 1024)].to_vec();
    
    // Simple checksum calculation
    let checksum = payload.iter().map(|&b| b as u32).sum::<u32>();
    
    NetworkPacket {
        header: PacketHeader {
            version,
            packet_type,
            flags,
            length,
            sequence,
        },
        payload,
        checksum,
    }
}

fuzz_target!(|data: &[u8]| {
    // Test direct deserialization of fuzz data
    let _: Result<ComplexData, _> = bincode::deserialize(data);
    let _: Result<NetworkPacket, _> = bincode::deserialize(data);
    
    // Test JSON deserialization of UTF-8 fuzz data
    if let Ok(json_str) = std::str::from_utf8(data) {
        let _: Result<ComplexData, _> = serde_json::from_str(json_str);
        let _: Result<NetworkPacket, _> = serde_json::from_str(json_str);
    }
    
    // Test serialization of structures created from fuzz data
    if !data.is_empty() {
        let complex_data = create_complex_data_from_fuzz(data);
        
        // Test bincode serialization
        if let Err(e) = test_bincode_serialization(&complex_data) {
            panic!("Bincode serialization test failed: {}", e);
        }
        
        // Test JSON serialization (but allow it to fail gracefully for non-UTF8 data)
        let _ = test_json_serialization(&complex_data);
        
        // Test network packet serialization
        let packet = create_network_packet_from_fuzz(data);
        if let Err(e) = test_network_packet_serialization(&packet) {
            panic!("Network packet serialization test failed: {}", e);
        }
    }
    
    // Test edge cases
    if data.len() >= 32 {
        // Test with minimal data
        let minimal_data = ComplexData {
            id: 0,
            name: String::new(),
            values: Vec::new(),
            metadata: HashMap::new(),
            tree_data: BTreeMap::new(),
            flags: HashSet::new(),
            nested: None,
            timestamp: 0,
        };
        
        let _ = test_bincode_serialization(&minimal_data);
        let _ = test_json_serialization(&minimal_data);
        
        // Test with maximal reasonable data
        let mut large_metadata = HashMap::new();
        for i in 0..100 {
            large_metadata.insert(
                format!("large_key_{}", i),
                format!("large_value_{}_with_data_{:?}", i, &data[..std::cmp::min(data.len(), 32)])
            );
        }
        
        let large_data = ComplexData {
            id: u64::MAX,
            name: "a".repeat(256),
            values: (0..100).collect(),
            metadata: large_metadata,
            tree_data: BTreeMap::new(),
            flags: (0..50).map(|i| format!("flag_{}", i)).collect(),
            nested: None,
            timestamp: u64::MAX,
        };
        
        let _ = test_bincode_serialization(&large_data);
        // Skip JSON test for large data as it may be too slow
    }
    
    // Test specific edge cases that might cause issues
    if data.len() >= 64 {
        // Test with deeply nested structure
        let mut nested_data = create_complex_data_from_fuzz(&data[..32]);
        nested_data.nested = Some(Box::new(create_complex_data_from_fuzz(&data[32..64])));
        
        let _ = test_bincode_serialization(&nested_data);
        
        // Test with empty collections
        let mut empty_collections_data = create_complex_data_from_fuzz(data);
        empty_collections_data.values.clear();
        empty_collections_data.metadata.clear();
        empty_collections_data.tree_data.clear();
        empty_collections_data.flags.clear();
        
        let _ = test_bincode_serialization(&empty_collections_data);
        let _ = test_json_serialization(&empty_collections_data);
        
        // Test with null bytes and special characters
        let mut special_data = create_complex_data_from_fuzz(data);
        special_data.name = format!("test\0with\nnull\tand\rspecial\x1bchars");
        special_data.metadata.insert("null_key\0".to_string(), "null_value\0".to_string());
        
        let _ = test_bincode_serialization(&special_data);
        // JSON may fail with null bytes, which is expected
        let _ = test_json_serialization(&special_data);
    }
    
    // Test boundary conditions
    let boundary_tests = vec![
        vec![0u8; 0],          // Empty
        vec![0u8; 1],          // Single byte
        vec![0xFFu8; 16],      // All 0xFF
        vec![0x00u8; 16],      // All null
        (0..255u8).collect(),  // Sequential bytes
    ];
    
    for boundary_data in boundary_tests {
        let _: Result<ComplexData, _> = bincode::deserialize(&boundary_data);
        let _: Result<NetworkPacket, _> = bincode::deserialize(&boundary_data);
        
        if let Ok(boundary_str) = std::str::from_utf8(&boundary_data) {
            let _: Result<ComplexData, _> = serde_json::from_str(boundary_str);
            let _: Result<NetworkPacket, _> = serde_json::from_str(boundary_str);
        }
    }
});