// 文件路径: /Thanatos/daemon/rust/src/event_logger.rs
use crate::grpc_generated::thanatos::ipc::EventLog;
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

pub struct EventLogger {
    conn: Arc<Mutex<Connection>>,
}

impl EventLogger {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn init_db(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS event_log (\n                id INTEGER PRIMARY KEY AUTOINCREMENT,\n                timestamp INTEGER NOT NULL,\n                package_name TEXT,\n                description TEXT NOT NULL\n            )",
            [],
        )?;
        Ok(())
    }

    pub fn log(&self, pkg_name: Option<&str>, description: &str) {
        let conn_guard = self.conn.lock().unwrap();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        if let Err(e) = conn_guard.execute(
            "INSERT INTO event_log (timestamp, package_name, description) VALUES (?1, ?2, ?3)",
            rusqlite::params![timestamp, pkg_name, description],
        ) {
            log::error!("Failed to write event log: {}", e);
        }
    }
    
    pub fn get_recent_events(&self, limit: u32) -> Result<Vec<EventLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT timestamp, package_name, description FROM event_log ORDER BY id DESC LIMIT ?1")?;
        let rows = stmt.query_map([limit], |row| {
            Ok(EventLog {
                timestamp: row.get(0)?,
                package_name: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                event_description: row.get(2)?,
            })
        })?;
        
        rows.collect::<Result<Vec<EventLog>>>()
    }
}
