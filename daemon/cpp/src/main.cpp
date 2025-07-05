// 文件路径: /Thanatos/daemon/cpp/src/main.cpp
#include <iostream>
#include <thread>
#include <memory>
#include <unistd.h>
#include <ev.h>

#include "grpc_server.h"
#include "timer_manager.h"
#include "thanatos_logic_bridge/src/ffi.rs.h"

// --- Global Variables ---
// The one and only instance of our Rust logic controller.
std::unique_ptr<LogicController> controller;
// The default libev event loop.
struct ev_loop *main_loop;

// --- Constants ---
const char* SOCKET_PATH = "/data/local/tmp/thanatosd.sock";
const char* DB_PATH = "/data/adb/modules/thanatos-core/thanatos.db";

// --- libev Callbacks ---
static ev_timer memory_check_watcher;

static void memory_check_callback(EV_P_ ev_timer *w, int revents) {
    if (!controller) return;
    rust_perform_memory_cleanup(*controller);
}

// --- Main Entry Point ---
int main(int argc, char** argv) {
    std::cout << "[C++ main] Starting thanatosd daemon..." << std::endl;

    // 1. Initialize Rust Logic Controller
    controller = rust_init_controller(DB_PATH);
    if (!controller) {
        std::cerr << "[C++ main] CRITICAL: Failed to initialize Rust controller. Exiting." << std::endl;
        return 1;
    }
    std::cout << "[C++ main] Rust controller initialized." << std::endl;

    // 2. Initialize libev event loop
    main_loop = EV_DEFAULT;
    if (!main_loop) {
        std::cerr << "[C++ main] CRITICAL: Could not initialize libev. Exiting." << std::endl;
        return 1;
    }

    // 3. Initialize Timer Manager
    init_timer_manager(main_loop);
    std::cout << "[C++ main] Timer manager initialized." << std::endl;

    // 4. Start the gRPC server in a separate thread
    // This is crucial because server->Wait() is a blocking call.
    std::thread grpc_thread(RunServer, std::string(SOCKET_PATH));
    grpc_thread.detach(); // Let the gRPC server run independently.
    std::cout << "[C++ main] gRPC server thread launched." << std::endl;
    
    // 5. Setup periodic tasks with libev
    // Setup memory cleanup task (e.g., every 60 seconds)
    ev_timer_init(&memory_check_watcher, memory_check_callback, 60.0, 60.0);
    ev_timer_start(main_loop, &memory_check_watcher);
    std::cout << "[C++ main] Periodic memory checker scheduled." << std::endl;

    // 6. Start the main event loop
    // This will block and process events (like timers firing) until ev_break is called.
    std::cout << "[C++ main] Starting main event loop. Daemon is now fully operational." << std::endl;
    ev_run(main_loop, 0);

    // This part of the code is unlikely to be reached unless the loop is explicitly broken.
    std::cout << "[C++ main] Event loop finished. Shutting down." << std::endl;
    
    return 0;
}
