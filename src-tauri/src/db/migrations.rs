use rusqlite::Connection;

pub fn initialize_schema(conn: &Connection) -> rusqlite::Result<()> {
    create_tables(conn)?;
    apply_migrations(conn)?;
    Ok(())
}

fn create_tables(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS llm_scan_configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            base_url TEXT NOT NULL DEFAULT '',
            api_key TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL DEFAULT '',
            provider TEXT NOT NULL DEFAULT '',
            api_version TEXT NOT NULL DEFAULT '',
            aws_region TEXT NOT NULL DEFAULT '',
            aws_profile TEXT NOT NULL DEFAULT '',
            aws_session_token TEXT NOT NULL DEFAULT '',
            max_tokens INTEGER NOT NULL DEFAULT 8192,
            consensus_runs INTEGER NOT NULL DEFAULT 1,
            is_active INTEGER NOT NULL DEFAULT 0,
            last_validated_at TEXT,
            last_validation_error TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_llm_config_active
            ON llm_scan_configs(is_active) WHERE is_active = 1;

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );
        ",
    )
}

fn apply_migrations(conn: &Connection) -> rusqlite::Result<()> {
    let version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version < 1 {
        conn.execute_batch("PRAGMA user_version = 1;")?;
    }
    if version < 2 {
        conn.execute_batch("PRAGMA user_version = 2;")?;
    }
    if version < 3 {
        migrate_v2_to_v3(conn)?;
    }
    let current: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if current < super::expected_schema_version() {
        conn.execute_batch(&format!(
            "PRAGMA user_version = {};",
            super::expected_schema_version()
        ))?;
    }
    Ok(())
}

fn migrate_v2_to_v3(conn: &Connection) -> rusqlite::Result<()> {
    let migrations = [
        ("api_version", "ALTER TABLE llm_scan_configs ADD COLUMN api_version TEXT NOT NULL DEFAULT ''"),
        ("aws_region", "ALTER TABLE llm_scan_configs ADD COLUMN aws_region TEXT NOT NULL DEFAULT ''"),
        ("aws_profile", "ALTER TABLE llm_scan_configs ADD COLUMN aws_profile TEXT NOT NULL DEFAULT ''"),
        ("aws_session_token", "ALTER TABLE llm_scan_configs ADD COLUMN aws_session_token TEXT NOT NULL DEFAULT ''"),
        ("last_validated_at", "ALTER TABLE llm_scan_configs ADD COLUMN last_validated_at TEXT"),
        ("last_validation_error", "ALTER TABLE llm_scan_configs ADD COLUMN last_validation_error TEXT NOT NULL DEFAULT ''"),
    ];

    for (column, sql) in migrations {
        if !table_has_column(conn, "llm_scan_configs", column)? {
            conn.execute_batch(sql)?;
        }
    }
    conn.execute_batch("PRAGMA user_version = 3;")?;
    Ok(())
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}
