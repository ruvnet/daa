use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create proto directory if it doesn't exist
    std::fs::create_dir_all("src/proto")?;
    
    // Check if proto files exist
    let proto_files = vec![
        "proto/gradient.proto",
        "proto/model.proto", 
        "proto/checkpoint.proto",
        "proto/messages.proto",
    ];
    
    let all_exist = proto_files.iter().all(|f| Path::new(f).exists());
    
    if all_exist {
        // Compile protobuf definitions
        prost_build::Config::new()
            .out_dir("src/proto")
            .compile_protos(&proto_files, &["proto/"])?;
    } else {
        println!("cargo:warning=Proto files not found, skipping protobuf compilation");
    }
    
    Ok(())
}