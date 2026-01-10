//! SQLite-backed idempotency store used by the SIRP server.
#[cfg(feature = "sqlite")]
use anyhow::{anyhow, Result};
#[cfg(feature = "sqlite")]
use chrono::Utc;
#[cfg(feature = "sqlite")]
use rusqlite::{params, Connection};
#[cfg(feature = "sqlite")]
use std::sync::Mutex;

/// Store `SQLite` de idempotência (P3).
#[cfg(feature = "sqlite")]
pub struct SqliteIdem {
    conn: Mutex<Connection>,
}
#[cfg(feature = "sqlite")]
impl SqliteIdem {
    /// Abre/cria o store.
    ///
    /// # Errors
    ///
    /// - Propaga erros de I/O ou `rusqlite` na abertura e migração do banco
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS processed (cid TEXT PRIMARY KEY, ts INTEGER NOT NULL);",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    /// Retorna true se já processado.
    ///
    /// # Errors
    ///
    /// - Propaga falhas de mutex ou de consulta no banco
    #[allow(clippy::significant_drop_tightening)]
    pub fn already(&self, cid: &str) -> Result<bool> {
        let exists;
        {
            let conn = self.conn.lock().map_err(|_| anyhow!("poisoned mutex"))?;
            let mut st = conn.prepare("SELECT 1 FROM processed WHERE cid=?1 LIMIT 1")?;
            exists = st.exists(params![cid])?;
        }
        Ok(exists)
    }
    /// Marca como processado.
    ///
    /// # Errors
    ///
    /// - Propaga falhas de mutex ou operações `rusqlite`
    pub fn mark(&self, cid: &str) -> Result<()> {
        let now = Utc::now().timestamp();
        self.conn
            .lock()
            .map_err(|_| anyhow!("poisoned mutex"))?
            .execute(
                "INSERT OR IGNORE INTO processed (cid, ts) VALUES (?1, ?2)",
                params![cid, now],
            )?;
        Ok(())
    }
    /// Remove entradas antigas.
    ///
    /// # Errors
    ///
    /// - Propaga falhas de mutex ou operações `rusqlite`
    pub fn cleanup_ttl_seconds(&self, ttl: i64) -> Result<usize> {
        let cutoff = Utc::now().timestamp() - ttl;
        self.conn
            .lock()
            .map_err(|_| anyhow!("poisoned mutex"))?
            .execute("DELETE FROM processed WHERE ts < ?1", params![cutoff])
            .map_err(Into::into)
    }
}
