// 文件路径: /Thanatos/daemon/rust/src/lib.rs

// 引入 ffi 模块定义
#[path = "ffi.rs"]
mod ffi;

// FFI 函数的具体实现
fn rust_layer_init() -> i32 {
    // 在这里可以进行 Rust 端的初始化，例如设置 logger
    // 为了简单起见，我们只打印一条消息
    println!("[Rust] Thanatos logic layer initialized.");
    0 // 返回 0 表示成功
}

fn rust_process_data(data: i32) -> i32 {
    println!("[Rust] Received data from C++: {}", data);
    let result = data + 10;
    println!("[Rust] Processed data, sending back: {}", result);
    result
}   