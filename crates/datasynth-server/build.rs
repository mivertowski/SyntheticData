use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/synth.proto");

    // Check if protoc is available
    let protoc_available = Command::new("protoc").arg("--version").output().is_ok();

    // Check if generated file already exists
    let generated_file = Path::new("src/grpc/synth.rs");
    let generated_exists = generated_file.exists()
        && std::fs::metadata(generated_file)
            .map(|m| m.len() > 100) // More than just a stub comment
            .unwrap_or(false);

    if !protoc_available && generated_exists {
        // Use existing pre-generated file (this is fine for release builds)
        println!("cargo:warning=protoc not found, using pre-generated proto code");
        return Ok(());
    }

    if !protoc_available {
        // FAIL the build - don't create a stub that leads to non-functional gRPC
        eprintln!();
        eprintln!("===========================================");
        eprintln!("ERROR: protoc (Protocol Buffers compiler) is required but not found.");
        eprintln!();
        eprintln!("The gRPC functionality requires protoc to compile .proto files.");
        eprintln!();
        eprintln!("To fix this:");
        eprintln!("  - Linux: apt install protobuf-compiler");
        eprintln!("  - macOS: brew install protobuf");
        eprintln!("  - Windows: choco install protoc");
        eprintln!();
        eprintln!("Or download from: https://github.com/protocolbuffers/protobuf/releases");
        eprintln!("===========================================");
        eprintln!();
        return Err(
            "protoc not found and no pre-generated gRPC code available. \
                    Install protoc or use pre-generated code from the repository."
                .into(),
        );
    }

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/grpc")
        .compile_protos(&["proto/synth.proto"], &["proto"])?;

    Ok(())
}
