// 文件路径: /Thanatos/daemon/cpp/src/timer_manager.cpp
#include "timer_manager.h"
#include <ev.h>
#include <map>
#include <mutex>
#include <iostream>

#include "thanatos_logic_bridge/src/ffi.rs.h"

// Forward declaration of the global Rust controller
extern std::unique_ptr<LogicController> controller;

// --- Static variables for timer management ---
static struct ev_loop* loop_ = nullptr;
static std::map<int64_t, ev_timer*> active_timers;
static std::mutex timers_mutex;
static int64_t next_timer_id = 1;

// The callback function executed by libev when a timer fires.
static void timer_callback(EV_P_ ev_timer *w, int revents) {
    // Retrieve the timer ID stored in the watcher's data field.
    int64_t timer_id = reinterpret_cast<int64_t>(w->data);
    
    // Notify the Rust layer that the timer has fired.
    if (controller) {
        rust_on_timer_fired(*controller, timer_id);
    }

    // Clean up resources for this one-shot timer.
    std::lock_guard<std::mutex> lock(timers_mutex);
    active_timers.erase(timer_id);
    ev_timer_stop(EV_A_ w);
    delete w; // Free the watcher memory.
}

void init_timer_manager(struct ev_loop* loop) {
    loop_ = loop;
}

int64_t create_oneshot_timer(int32_t duration_ms) {
    if (!loop_) {
        std::cerr << "[C++ Timer] Error: Timer manager not initialized!" << std::endl;
        return 0;
    }
    
    std::lock_guard<std::mutex> lock(timers_mutex);
    
    int64_t timer_id = next_timer_id++;
    ev_timer* timer_watcher = new ev_timer();
    timer_watcher->data = reinterpret_cast<void*>(timer_id);
    
    double duration_sec = static_cast<double>(duration_ms) / 1000.0;
    
    ev_timer_init(timer_watcher, timer_callback, duration_sec, 0.);
    ev_timer_start(loop_, timer_watcher);

    active_timers[timer_id] = timer_watcher;
    std::cout << "[C++ Timer] Created timer with ID " << timer_id << " for " << duration_ms << "ms" << std::endl;
    
    return timer_id;
}

void cancel_timer(int64_t timer_id) {
    std::lock_guard<std::mutex> lock(timers_mutex);
    auto it = active_timers.find(timer_id);
    if (it != active_timers.end()) {
        ev_timer_stop(loop_, it->second);
        delete it->second;
        active_timers.erase(it);
        std::cout << "[C++ Timer] Canceled timer with ID " << timer_id << std::endl;
    }
}

// --- Implementation of functions callable by Rust via CXX ---

// These functions are the C++ side of the FFI bridge defined in `ffi.rs`.
// They simply delegate to the internal timer management functions.
int64_t cpp_create_oneshot_timer(int32_t duration_ms) {
    return create_oneshot_timer(duration_ms);
}

void cpp_cancel_timer(int64_t timer_id) {
    cancel_timer(timer_id);
}

// We need to declare and implement all C++ functions called by Rust.
// We'll put them all here for simplicity.
#include "kernel_interface.h"
#include "shell_interface.h"

void cpp_set_oom_score_adj(int32_t pid, int32_t score) {
    set_oom_score_adj(pid, score);
}

int64_t cpp_get_available_memory_kb() {
    return get_available_memory_kb();
}

int64_t cpp_get_total_memory_kb() {
    return get_total_memory_kb();
}

float cpp_get_cpu_usage_percent() {
    return get_cpu_usage_percent();
}

rust::String cpp_execute_shell(rust::Str command) {
    return execute_shell_command(std::string(command));
}
