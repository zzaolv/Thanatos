// 文件路径: /Thanatos/daemon/rust/src/config_manager.rs
use crate::grpc_generated::thanatos::ipc::{AppConfig, LaunchRule};
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

pub struct ConfigManager {
    conn: Arc<Mutex<Connection>>,
}

impl ConfigManager {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let config_manager = ConfigManager {
            conn: Arc::new(Mutex::new(conn)),
        };
        config_manager.init_db()?;
        Ok(config_manager)
    }

    fn init_db(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // App configurations table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_configs (\n                package_name TEXT PRIMARY KEY,\n                policy INTEGER NOT NULL,\n                freeze_mode INTEGER NOT NULL,\n                oom_priority INTEGER NOT NULL,\n                network_policy INTEGER NOT NULL,\n                allow_wakeup_for_push BOOLEAN NOT NULL,\n                allow_autostart BOOLEAN NOT NULL\n            )",
            [],
        )?;
        // Launch rules table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS launch_rules (\n                source_package TEXT NOT NULL,\n                target_package TEXT NOT NULL,\n                allowed BOOLEAN NOT NULL,\n                PRIMARY KEY (source_package, target_package)\n            )",
            [],
        )?;
        Ok(())
    }

    pub fn set_app_config(&self, config: &AppConfig) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO app_configs (\n                package_name, policy, freeze_mode, oom_priority, network_policy,\n                allow_wakeup_for_push, allow_autostart\n            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                config.package_name,
                config.policy,
                config.freeze_mode,
                config.oom_priority,
                config.network_policy,
                config.allow_wakeup_for_push,
                config.allow_autostart,
            ],
        )?;
        Ok(())
    }

    pub fn get_app_config(&self, package_name: &str) -> Result<Option<AppConfig>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM app_configs WHERE package_name = ?1")?;
        let mut rows = stmt.query_map([package_name], |row| {
            Ok(AppConfig {
                package_name: row.get(0)?,
                policy: row.get(1)?,
                freeze_mode: row.get(2)?,
                oom_priority: row.get(3)?,
                network_policy: row.get(4)?,
                allow_wakeup_for_push: row.get(5)?,
                allow_autostart: row.get(6)?,
            })
        })?;

        rows.next().transpose().map_err(|e| e.into())
    }

    pub fn set_launch_rule(&self, rule: &LaunchRule) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO launch_rules (source_package, target_package, allowed) VALUES (?1, ?2, ?3)",
            rusqlite::params![rule.source_package, rule.target_package, rule.allowed],
        )?;
        Ok(())
    }
    
    pub fn get_launch_rule(&self, source: &str, target: &str) -> Result<Option<bool>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT allowed FROM launch_rules WHERE source_package = ?1 AND target_package = ?2")?;
        stmt.query_row([source, target], |row| row.get(0)).map(Some).or_else(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                Ok(None)
            } else {
                Err(e)
            }
        })
    }
}
