use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create proto directory if it doesn't exist
    std::fs::create_dir_all("src/proto")?;
    
    // Check if protoc is available
    let protoc_available = Command::new("protoc")
        .arg("--version")
        .output()
        .is_ok();
    
    // Check if proto files exist
    let proto_files = vec![
        "proto/gradient.proto",
        "proto/model.proto", 
        "proto/checkpoint.proto",
        "proto/messages.proto",
    ];
    
    let all_proto_exist = proto_files.iter().all(|f| Path::new(f).exists());
    
    // Check if compiled proto files already exist
    let compiled_files = vec![
        "src/proto/gradient.rs",
        "src/proto/model.rs",
        "src/proto/checkpoint.rs", 
        "src/proto/messages.rs",
    ];
    
    let all_compiled_exist = compiled_files.iter().all(|f| Path::new(f).exists());
    
    if protoc_available && all_proto_exist {
        // Compile protobuf definitions
        prost_build::Config::new()
            .out_dir("src/proto")
            .compile_protos(&proto_files, &["proto/"])?;
        println!("cargo:warning=Compiled proto files with protoc");
    } else if all_compiled_exist {
        println!("cargo:warning=Using pre-compiled proto files (protoc not available)");
    } else {
        println!("cargo:warning=Neither protoc nor pre-compiled proto files available");
    }
    
    Ok(())
}