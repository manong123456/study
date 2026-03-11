use tokio_rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use anyhow::Result;

pub async fn init_db(db_path: &str) -> Result<Connection> {
    tokio::fs::create_dir_all(db_path).await?;
    let db_file = format!("{}/capeos.db", db_path);
    let conn = Connection::open(&db_file).await?;

    conn.call(|conn| -> Result<(), rusqlite_migration::Error> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "busy_timeout", "5000")?;

        let migrations = Migrations::new(vec![
            M::up("CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                role TEXT DEFAULT 'user',
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now'))
            );"),
        ]);
        migrations.to_latest(conn)?;
        Ok(())
    }).await?;

    Ok(conn)
}
