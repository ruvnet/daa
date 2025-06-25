fn main() {
    // Set up WASM target configuration
    if std::env::var("TARGET").unwrap_or_default().contains("wasm32") {
        println!("cargo:rustc-link-arg=--export-table");
        println!("cargo:rustc-link-arg=-zstack-size=1048576");
    }
}