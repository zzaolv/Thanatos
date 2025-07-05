// 文件路径: /Thanatos/daemon/rust/src/execution_manager.rs
use crate::grpc_generated::thanatos::ipc::FreezeMode;

pub fn freeze_app(pid: i32, mode: FreezeMode) {
    if pid <= 0 {
        log::warn!("Invalid PID {} for freezing", pid);
        return;
    }
    log::info!("Executing freeze for PID: {} with mode {:?}", pid, mode);
    unsafe {
        match mode {
            FreezeMode::ModeSigstop => {
                libc::kill(pid, libc::SIGSTOP);
            }
            FreezeMode::ModeKill => {
                libc::kill(pid, libc::SIGKILL);
            }
            FreezeMode::ModeHibernate => {
                // This requires executing a shell command, which is handled in C++
                log::warn!("Hibernate mode not fully implemented in Rust execution manager. Use shell interface.");
                libc::kill(pid, libc::SIGSTOP); // Fallback
            }
            FreezeMode::ModeUnspecified => {
                log::warn!("Unspecified freeze mode for PID {}, defaulting to SIGSTOP", pid);
                libc::kill(pid, libc::SIGSTOP);
            }
        }
    }
}

pub fn unfreeze_app(pid: i32) {
    if pid <= 0 {
        log::warn!("Invalid PID {} for unfreezing", pid);
        return;
    }
    log::info!("Executing unfreeze for PID: {}", pid);
    unsafe {
        libc::kill(pid, libc::SIGCONT);
    }
}
