// 文件路径: /Thanatos/daemon/rust/src/logic_controller.rs

use crate::config_manager::ConfigManager;
use crate::event_logger::EventLogger;
use crate::execution_manager::{freeze_app, unfreeze_app};
use crate::ffi;
use crate::grpc_generated::thanatos::ipc::{self as grpc_ipc};
use crate::ml_collector::MLDataCollector;

use dashmap::DashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum AppState {
    Foreground(i32), // pid
    Background(i32), // pid
    Frozen(i32),     // pid
    TemporarilyUnfrozen { pid: i32, refreeze_timer_id: i64 },
}

pub struct LogicController {
    config_manager: Arc<ConfigManager>,
    event_logger: Arc<EventLogger>,
    ml_collector: Arc<MLDataCollector>,
    runtime_states: Arc<DashMap<String, AppState>>,
    smart_standby_timer_id: Arc<Mutex<Option<i64>>>,
    background_lru: Arc<Mutex<VecDeque<String>>>,
    foreground_package: Arc<Mutex<String>>,
}

impl LogicController {
    pub fn new(
        config_manager: Arc<ConfigManager>,
        event_logger: Arc<EventLogger>,
        ml_collector: Arc<MLDataCollector>,
    ) -> Self {
        Self {
            config_manager,
            event_logger,
            ml_collector,
            runtime_states: Arc::new(DashMap::new()),
            smart_standby_timer_id: Arc::new(Mutex::new(None)),
            background_lru: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            foreground_package: Arc::new(Mutex::new(String::new())),
        }
    }

    // --- Public API called from FFI ---

    pub fn set_app_config(&self, config: grpc_ipc::AppConfig) {
        let pkg = config.package_name.clone();
        if let Err(e) = self.config_manager.set_app_config(&config) {
            self.event_logger.log(Some(&pkg), &format!("Error setting config: {}", e));
        } else {
            self.event_logger.log(Some(&pkg), "Configuration updated");
        }
    }

    pub fn set_launch_rule(&self, rule: grpc_ipc::LaunchRule) {
        if let Err(e) = self.config_manager.set_launch_rule(&rule) {
            log::error!("Failed to set launch rule: {}", e);
        }
    }

    pub fn handle_framework_event(&self, event: grpc_ipc::FrameworkEvent) {
        match event.event_type() {
            grpc_ipc::framework_event::EventType::AppForeground => self.handle_app_state_change(event, true),
            grpc_ipc::framework_event::EventType::AppBackground => self.handle_app_state_change(event, false),
            grpc_ipc::framework_event::EventType::ScreenOn => self.handle_screen_on(),
            grpc_ipc::framework_event::EventType::ScreenOff => self.handle_screen_off(),
            _ => {}
        }
    }
    
    pub fn request_temporary_unfreeze(&self, package_name: &str, reason: grpc_ipc::UnfreezeReason, duration_ms: i32) {
        if let Some(mut state_entry) = self.runtime_states.get_mut(package_name) {
            if let AppState::Frozen(pid) = *state_entry {
                log::info!("Unfreezing '{}' (PID: {}) temporarily.", package_name, pid);
                unfreeze_app(pid);
                self.event_logger.log(Some(package_name), &format!("Temporarily unfrozen for {:?}", reason));

                let timer_id = unsafe { ffi::cpp_create_oneshot_timer(duration_ms) };
                *state_entry = AppState::TemporarilyUnfrozen { pid, refreeze_timer_id: timer_id };
            }
        }
    }

    pub fn should_allow_launch(&self, rule: &grpc_ipc::LaunchRule) -> grpc_ipc::LaunchRule {
        let mut result_rule = rule.clone();
        match self.config_manager.get_launch_rule(&rule.source_package, &rule.target_package) {
            Ok(Some(allowed)) => result_rule.allowed = allowed,
            Ok(None) => result_rule.allowed = true, // Default allow
            Err(e) => {
                log::error!("DB error on get_launch_rule: {}", e);
                result_rule.allowed = true; // Fail-safe
            }
        }
        if !result_rule.allowed {
            self.event_logger.log(Some(&rule.source_package), &format!("Blocked launch towards {}", rule.target_package));
        }
        result_rule
    }

    pub fn on_timer_fired(&self, timer_id: i64) {
        if self.handle_standby_timer(timer_id) { return; }
        self.handle_refreeze_timer(timer_id);
    }
    
    pub fn perform_memory_cleanup(&self) {
        let avail_mem_kb = unsafe { ffi::cpp_get_available_memory_kb() };
        let threshold_kb: i64 = 500 * 1024; // 500MB, should be configurable

        if avail_mem_kb < threshold_kb {
            log::warn!("Low memory detected ({avail_mem_kb}KB < {threshold_kb}KB). Starting cleanup.");
            self.event_logger.log(None, "Low memory detected, starting cleanup.");
            
            let lru = self.background_lru.lock().unwrap();
            for pkg_name in lru.iter().rev().take(3) { // Clean up to 3 apps
                if let Some(state) = self.runtime_states.get(pkg_name) {
                    if let AppState::Background(pid) | AppState::Frozen(pid) = *state.value() {
                        // TODO: Check config for OOM_LOW priority
                        self.event_logger.log(Some(pkg_name), "Killed due to low memory");
                        freeze_app(pid, grpc_ipc::FreezeMode::ModeKill);
                    }
                }
            }
        }
    }
    
    pub fn get_runtime_stats(&self) -> ffi::RustRuntimeStats {
        let frozen_count = self.runtime_states.iter().filter(|e| matches!(*e.value(), AppState::Frozen(_))).count();
        ffi::RustRuntimeStats {
            total_mem_kb: unsafe { ffi::cpp_get_total_memory_kb() },
            avail_mem_kb: unsafe { ffi::cpp_get_available_memory_kb() },
            frozen_app_count: frozen_count as i32,
            cpu_usage_percent: unsafe { ffi::cpp_get_cpu_usage_percent() },
            network_speed_bps: 0, // Not implemented yet
        }
    }
    
    pub fn get_recent_events(&self, limit: u32) -> Vec<grpc_ipc::EventLog> {
        self.event_logger.get_recent_events(limit).unwrap_or_default()
    }

    // --- Internal Logic ---

    fn handle_app_state_change(&self, event: grpc_ipc::FrameworkEvent, is_foreground: bool) {
        let pkg_name = event.target_package;
        let pid = event.source_package.parse::<i32>().unwrap_or(0);
        if pkg_name.is_empty() || pid <= 0 { return; }

        let old_state = self.runtime_states.get(&pkg_name).map(|r| r.value().clone());

        if is_foreground {
            *self.foreground_package.lock().unwrap() = pkg_name.clone();
            self.event_logger.log(Some(&pkg_name), "App became foreground");
            // Cancel any pending timers for this app
            if let Some(AppState::TemporarilyUnfrozen { refreeze_timer_id, .. }) = old_state {
                unsafe { ffi::cpp_cancel_timer(refreeze_timer_id) };
            }
            unfreeze_app(pid);
            self.runtime_states.insert(pkg_name.clone(), AppState::Foreground(pid));
            
            // AI Data Collection
            if let Some(AppState::Frozen(_)) = old_state {
                let context = self.foreground_package.lock().unwrap().clone();
                self.ml_collector.log_training_data(Some(&context), &pkg_name, "USER_LAUNCH_AFTER_FREEZE", 1);
            }
        } else {
            self.event_logger.log(Some(&pkg_name), "App became background");
            self.runtime_states.insert(pkg_name.clone(), AppState::Background(pid));
            self.add_to_lru(&pkg_name);
            self.adjust_oom_score(&pkg_name, pid);
            
            // Check policy and freeze if necessary
            if let Ok(Some(config)) = self.config_manager.get_app_config(&pkg_name) {
                if config.policy == grpc_ipc::ManagementPolicy::StrictBackground as i32 {
                    self.event_logger.log(Some(&pkg_name), "Frozen by STRICT_BACKGROUND policy");
                    freeze_app(pid, config.freeze_mode.into());
                    self.runtime_states.insert(pkg_name, AppState::Frozen(pid));
                }
            }
        }
    }

    fn handle_screen_on(&self) {
        self.event_logger.log(None, "Screen ON");
        if let Some(timer_id) = self.smart_standby_timer_id.lock().unwrap().take() {
            unsafe { ffi::cpp_cancel_timer(timer_id) };
            self.event_logger.log(None, "Smart Standby cancelled");
        }
    }

    fn handle_screen_off(&self) {
        self.event_logger.log(None, "Screen OFF");
        let standby_delay_ms = 300 * 1000; // 5 minutes, should be configurable
        let timer_id = unsafe { ffi::cpp_create_oneshot_timer(standby_delay_ms) };
        *self.smart_standby_timer_id.lock().unwrap() = Some(timer_id);
        self.event_logger.log(None, &format!("Smart Standby scheduled in {}s", standby_delay_ms / 1000));
    }

    fn handle_standby_timer(&self, timer_id: i64) -> bool {
        let mut id_lock = self.smart_standby_timer_id.lock().unwrap();
        if id_lock.map_or(false, |id| id == timer_id) {
            *id_lock = None;
            self.event_logger.log(None, "Smart Standby triggered, freezing background apps");
            for mut entry in self.runtime_states.iter_mut() {
                if let AppState::Background(pid) = *entry.value() {
                    // TODO: Check for standby-exempt apps
                    freeze_app(pid, grpc_ipc::FreezeMode::ModeSigstop);
                    *entry.value_mut() = AppState::Frozen(pid);
                }
            }
            return true;
        }
        false
    }

    fn handle_refreeze_timer(&self, timer_id: i64) {
        let pkg_to_refreeze = self.runtime_states.iter()
            .find(|entry| matches!(*entry.value(), AppState::TemporarilyUnfrozen { refreeze_timer_id, .. } if refreeze_timer_id == timer_id))
            .map(|entry| entry.key().clone());
            
        if let Some(pkg_name) = pkg_to_refreeze {
            if let Some(mut state) = self.runtime_states.get_mut(&pkg_name) {
                if let AppState::TemporarilyUnfrozen { pid, .. } = *state.value() {
                    self.event_logger.log(Some(&pkg_name), "Re-freezing after temporary exemption");
                    freeze_app(pid, grpc_ipc::FreezeMode::ModeSigstop);
                    *state.value_mut() = AppState::Frozen(pid);
                }
            }
        }
    }

    fn add_to_lru(&self, pkg_name: &str) {
        let mut lru = self.background_lru.lock().unwrap();
        lru.retain(|p| p != pkg_name);
        lru.push_front(pkg_name.to_string());
        if lru.len() > 100 { lru.pop_back(); }
    }

    fn adjust_oom_score(&self, pkg_name: &str, pid: i32) {
        if pid <= 0 { return; }
        let lru = self.background_lru.lock().unwrap();
        let position = lru.iter().position(|p| p == pkg_name).unwrap_or(lru.len());
        
        let base_score = 200;
        let score_step = 50;
        let oom_prio_factor = match self.config_manager.get_app_config(pkg_name) {
            Ok(Some(config)) => match config.oom_priority() {
                grpc_ipc::OomPriority::OomHigh => -2, // less likely to be killed
                grpc_ipc::OomPriority::OomLow => 2,   // more likely to be killed
                _ => 0,
            },
            _ => 0,
        };
        
        let score = base_score + (position as i32 * score_step) + (oom_prio_factor * 100);
        let final_score = std::cmp::min(score, 900); // Cap at 900
        unsafe { ffi::cpp_set_oom_score_adj(pid, final_score) };
    }
}
