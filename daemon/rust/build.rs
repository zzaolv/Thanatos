// 文件路径: /Thanatos/daemon/rust/build.rs

fn main() {
    cxx_build::bridge("src/ffi.rs") // 指定哪个文件包含了 cxx::bridge
        .file("src/lib.rs")      // 指定哪个文件包含了 FFI 实现
        .flag_if_supported("-std=c++17")
        .compile("thanatos_logic_bridge");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/ffi.rs");
    // 如果有其他 rust 源文件，也需要在这里声明
}