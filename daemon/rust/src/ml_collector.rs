// 文件路径: /Thanatos/daemon/rust/src/ml_collector.rs
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

pub struct MLDataCollector {
    conn: Arc<Mutex<Connection>>,
}

impl MLDataCollector {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
    
    pub fn init_db(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ai_training_data (\n                id INTEGER PRIMARY KEY AUTOINCREMENT,\n                timestamp INTEGER NOT NULL,\n                context_pkg TEXT,      -- The app that was in foreground\n                target_pkg TEXT NOT NULL,  -- The app the user interacted with\n                event_type TEXT NOT NULL,  -- e.g., 'USER_LAUNCH_AFTER_FREEZE'\n                label INTEGER NOT NULL     -- e.g., 1 for positive, 0 for negative\n            )",
            [],
        )?;
        Ok(())
    }
    
    pub fn log_training_data(&self, context_pkg: Option<&str>, target_pkg: &str, event_type: &str, label: i32) {
        let conn = self.conn.lock().unwrap();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
            
        if let Err(e) = conn.execute(
            "INSERT INTO ai_training_data (timestamp, context_pkg, target_pkg, event_type, label) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![timestamp, context_pkg, target_pkg, event_type, label],
        ) {
            log::error!("Failed to log AI training data: {}", e);
        } else {
            log::info!("Logged AI training data for event: {}", event_type);
        }
    }
}
