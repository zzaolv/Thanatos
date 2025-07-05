// 文件路径: /Thanatos/daemon/rust/build.rs

fn main() {
    // 1. Compile Protobuf/gRPC definitions for Rust.
    // This generates Rust types and service traits from the .proto file.
    let proto_path = "../../ipc/proto/thanatos.proto";
    println!("cargo:rerun-if-changed={}", proto_path);
    tonic_build::configure()
        .build_server(true)  // We need server traits for type definitions.
        .build_client(false) // We don't implement a Rust gRPC client in the daemon.
        .out_dir("src/grpc_generated") // Output to a dedicated directory.
        .compile(&[proto_path], &["../../ipc/proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos: {}", e));

    // 2. Build the CXX FFI bridge.
    // This generates the C++ header and the Rust-side FFI implementation.
    cxx_build::bridge("src/ffi.rs") // The file with the #[cxx::bridge] module.
        // List all Rust files that contain `extern "Rust"` function implementations.
        .file("src/lib.rs")
        .file("src/logic_controller.rs")
        .file("src/config_manager.rs")
        .file("src/event_logger.rs")
        .file("src/execution_manager.rs")
        .file("src/ml_collector.rs")
        .flag_if_supported("-std=c++17")
        .compile("thanatos_logic_bridge");

    // Tell Cargo when to rerun this build script.
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=src/logic_controller.rs");
    println!("cargo:rerun-if-changed=src/config_manager.rs");
    println!("cargo:rerun-if-changed=src/event_logger.rs");
    println!("cargo:rerun-if-changed=src/execution_manager.rs");
    println!("cargo:rerun-if-changed=src/ml_collector.rs");
}
