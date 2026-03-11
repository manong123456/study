//! Database initialization and migrations for the user service.
//!
//! Creates the SQLite database and users table with Argon2-hashed passwords.

use anyhow::Result;
use rusqlite_migration::{Migrations, M};
use tokio_rusqlite::Connection;

/// Initializes the SQLite database at the given path.
///
/// Creates the directory if needed, opens `capeos.db`, and applies migrations.
/// Sets PRAGMA `journal_mode=WAL` and `busy_timeout=5000` for better concurrency.
/// Creates the `users` table with id, username, password_hash, role, created_at, updated_at.
///
/// # Arguments
///
/// * `db_path` - Directory path where the database file will be created (e.g. `/var/lib/capeos/db`)
///
/// # Returns
///
/// A tokio-rusqlite `Connection` to the initialized database.
pub async fn init_db(db_path: &str) -> Result<Connection> {
    tokio::fs::create_dir_all(db_path).await?;
    let db_file = format!("{}/capeos.db", db_path);
    let conn = Connection::open(&db_file).await?;

    conn.call(|conn| -> Result<(), rusqlite_migration::Error> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "busy_timeout", "5000")?;

        let migrations = Migrations::new(vec![M::up(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                role TEXT DEFAULT 'user',
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now'))
            );",
        )]);
        migrations.to_latest(conn)?;
        Ok(())
    })
    .await?;

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_db() {
        let dir = format!("/tmp/capeos_test_db_{}", std::process::id());
        let conn = init_db(&dir).await.unwrap();
        let db_file = format!("{}/capeos.db", dir);
        assert!(std::path::Path::new(&db_file).exists());
        let exists: bool = conn
            .call(|conn| -> Result<bool, rusqlite::Error> {
                let count: i32 = conn.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'",
                    [],
                    |row| row.get(0),
                )?;
                Ok(count > 0)
            })
            .await
            .unwrap();
        assert!(exists);
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}
