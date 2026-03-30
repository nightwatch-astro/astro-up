use rusqlite::{Connection, OptionalExtension, params};

/// SQLite-backed config key-value store.
pub struct ConfigStore {
    conn: Connection,
}

impl ConfigStore {
    /// Open or create the config store. Creates the `config_settings` table if it doesn't exist.
    pub fn new(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS config_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    /// Get a stored value by key. Returns `None` if the key was never set.
    pub fn get(&self, key: &str) -> rusqlite::Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT value FROM config_settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
    }

    /// Set a value. Inserts or replaces if the key already exists.
    pub fn set(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO config_settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
            params![key, value],
        )?;
        Ok(())
    }

    /// List all stored key-value pairs, ordered by key.
    pub fn list(&self) -> rusqlite::Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM config_settings ORDER BY key")?;
        let pairs = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(pairs)
    }

    /// Remove a stored override, reverting the key to its default.
    pub fn reset(&self, key: &str) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM config_settings WHERE key = ?1", params![key])?;
        Ok(())
    }

    /// Remove all stored overrides.
    pub fn reset_all(&self) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM config_settings", [])?;
        Ok(())
    }
}
