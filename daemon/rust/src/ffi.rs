// 文件路径: /Thanatos/daemon/rust/src/ffi.rs

#[cxx::bridge(namespace = "thanatos")]
pub mod ffi {
    // --- Structs for safe data transfer between C++ and Rust ---
    #[derive(Default)]
    struct CxxAppConfig {
        package_name: String,
        policy: i32,
        freeze_mode: i32,
        oom_priority: i32,
        network_policy: i32,
        allow_wakeup_for_push: bool,
        allow_autostart: bool,
    }

    #[derive(Default)]
    struct CxxLaunchRule {
        source_package: String,
        target_package: String,
        allowed: bool,
    }
    
    #[derive(Default)]
    struct CxxFrameworkEvent {
        event_type: i32,
        source_package: String,
        target_package: String,
        timestamp: i64,
    }

    #[derive(Default)]
    struct RustRuntimeStats {
        total_mem_kb: i64,
        avail_mem_kb: i64,
        frozen_app_count: i32,
        cpu_usage_percent: f32,
        network_speed_bps: i64,
    }
    
    #[derive(Default)]
    struct CxxEventLog {
        timestamp: i64,
        package_name: String,
        description: String,
    }

    // --- Rust functions exposed to C++ ---
    extern "Rust" {
        type LogicController;

        // Lifecycle
        fn rust_init_controller(db_path: &str) -> UniquePtr<LogicController>;

        // App Control
        fn rust_set_app_config(controller: &LogicController, config: CxxAppConfig);
        fn rust_set_launch_rule(controller: &LogicController, rule: CxxLaunchRule);

        // System Events & Decisions
        fn rust_handle_framework_event(controller: &LogicController, event: CxxFrameworkEvent);
        fn rust_request_temporary_unfreeze(controller: &LogicController, package_name: &str, reason: i32, duration_ms: i32);
        fn rust_should_allow_launch(controller: &LogicController, rule: CxxLaunchRule) -> CxxLaunchRule;
        fn rust_on_timer_fired(controller: &LogicController, timer_id: i64);
        fn rust_perform_memory_cleanup(controller: &LogicController);

        // Dashboard
        fn rust_get_runtime_stats(controller: &LogicController) -> RustRuntimeStats;
        fn rust_get_recent_events(controller: &LogicController, limit: u32) -> Vec<CxxEventLog>;
    }

    // --- C++ functions callable from Rust ---
    unsafe extern "C++" {
        // Timers
        fn cpp_create_oneshot_timer(duration_ms: i32) -> i64;
        fn cpp_cancel_timer(timer_id: i64);

        // Kernel Interface
        fn cpp_set_oom_score_adj(pid: i32, score: i32);
        fn cpp_get_available_memory_kb() -> i64;
        fn cpp_get_total_memory_kb() -> i64;
        fn cpp_get_cpu_usage_percent() -> f32;
        
        // Shell
        fn cpp_execute_shell(command: &str) -> String;
    }
}
