// 文件路径: /Thanatos/daemon/rust/src/ffi.rs

#[cxx::bridge]
mod ffi {
    // Rust 侧的函数，暴露给 C++
    extern "Rust" {
        /// 初始化 Rust 逻辑层，返回 0 表示成功
        fn rust_layer_init() -> i32;

        /// 一个简单的测试函数，将传入的数字加 10 并返回
        fn rust_process_data(data: i32) -> i32;
    }

    // C++ 侧的函数，可由 Rust 调用 (暂时为空)
    unsafe extern "C++" {
        // include!("path/to/cpp_header.h");
        // fn cpp_function_callable_from_rust();
    }
}