use serde::Serialize;

use crate::db::Database;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmScanConfigRow {
    pub id: Option<i64>,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub provider: String,
    pub api_version: String,
    pub aws_region: String,
    pub aws_profile: String,
    pub aws_session_token: String,
    pub max_tokens: i64,
    pub consensus_runs: i64,
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_validated_at: Option<String>,
    pub last_validation_error: String,
}

const CONFIG_COLUMNS: &str = "id, name, base_url, api_key, model, provider, api_version, aws_region, \
    aws_profile, aws_session_token, max_tokens, consensus_runs, is_active, \
    last_validated_at, last_validation_error";

#[derive(Clone)]
pub struct ScanConfigRepository {
    db: std::sync::Arc<Database>,
}

impl ScanConfigRepository {
    pub fn new(db: std::sync::Arc<Database>) -> Self {
        Self { db }
    }

    pub fn list_all(&self) -> Result<Vec<LlmScanConfigRow>, crate::db::DatabaseError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT {CONFIG_COLUMNS} FROM llm_scan_configs ORDER BY id"
            ))?;
            let rows = stmt
                .query_map([], |row| row_to_config(row))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn get_active(&self) -> Result<Option<LlmScanConfigRow>, crate::db::DatabaseError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT {CONFIG_COLUMNS} FROM llm_scan_configs WHERE is_active = 1"
            ))?;
            let mut rows = stmt.query([])?;
            if let Some(row) = rows.next()? {
                return row_to_config(row).map(Some);
            }
            Ok(None)
        })
    }

    pub fn get_by_id(&self, config_id: i64) -> Result<Option<LlmScanConfigRow>, crate::db::DatabaseError> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT {CONFIG_COLUMNS} FROM llm_scan_configs WHERE id = ?1"
            ))?;
            let mut rows = stmt.query(rusqlite::params![config_id])?;
            if let Some(row) = rows.next()? {
                return row_to_config(row).map(Some);
            }
            Ok(None)
        })
    }

    pub fn save(&self, config: &LlmScanConfigRow) -> Result<i64, crate::db::DatabaseError> {
        self.db.with_connection(|conn| {
            if let Some(id) = config.id {
                conn.execute(
                    "UPDATE llm_scan_configs
                     SET name=?1, base_url=?2, api_key=?3, model=?4, provider=?5,
                         api_version=?6, aws_region=?7, aws_profile=?8, aws_session_token=?9,
                         max_tokens=?10, consensus_runs=?11, is_active=?12,
                         last_validated_at=?13, last_validation_error=?14,
                         updated_at=datetime('now')
                     WHERE id=?15",
                    rusqlite::params![
                        config.name,
                        config.base_url,
                        config.api_key,
                        config.model,
                        config.provider,
                        config.api_version,
                        config.aws_region,
                        config.aws_profile,
                        config.aws_session_token,
                        config.max_tokens,
                        config.consensus_runs,
                        config.is_active as i64,
                        config.last_validated_at,
                        config.last_validation_error,
                        id,
                    ],
                )?;
                Ok(id)
            } else {
                conn.query_row(
                    "INSERT INTO llm_scan_configs (
                        name, base_url, api_key, model, provider,
                        api_version, aws_region, aws_profile, aws_session_token,
                        max_tokens, consensus_runs, is_active, last_validated_at, last_validation_error
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                    RETURNING id",
                    rusqlite::params![
                        config.name,
                        config.base_url,
                        config.api_key,
                        config.model,
                        config.provider,
                        config.api_version,
                        config.aws_region,
                        config.aws_profile,
                        config.aws_session_token,
                        config.max_tokens,
                        config.consensus_runs,
                        config.is_active as i64,
                        config.last_validated_at,
                        config.last_validation_error,
                    ],
                    |row| row.get(0),
                )
            }
        })
    }

    pub fn delete(&self, config_id: i64) -> Result<(), crate::db::DatabaseError> {
        self.db
            .execute("DELETE FROM llm_scan_configs WHERE id = ?1", rusqlite::params![config_id])
            .map(|_| ())
    }

    pub fn set_active(&self, config_id: i64) -> Result<(), crate::db::DatabaseError> {
        self.db.with_connection(|conn| {
            conn.execute(
                "UPDATE llm_scan_configs SET is_active = 0 WHERE is_active = 1",
                [],
            )?;
            conn.execute(
                "UPDATE llm_scan_configs SET is_active = 1, updated_at=datetime('now') WHERE id = ?1",
                rusqlite::params![config_id],
            )?;
            Ok(())
        })
    }
}

fn row_to_config(row: &rusqlite::Row<'_>) -> rusqlite::Result<LlmScanConfigRow> {
    Ok(LlmScanConfigRow {
        id: row.get(0)?,
        name: row.get(1)?,
        base_url: row.get(2)?,
        api_key: row.get(3)?,
        model: row.get(4)?,
        provider: row.get(5)?,
        api_version: row.get(6)?,
        aws_region: row.get(7)?,
        aws_profile: row.get(8)?,
        aws_session_token: row.get(9)?,
        max_tokens: row.get(10)?,
        consensus_runs: row.get(11)?,
        is_active: row.get::<_, i64>(12)? != 0,
        last_validated_at: row.get(13)?,
        last_validation_error: row.get(14)?,
    })
}

pub fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    if key.len() <= 8 {
        return "****".to_string();
    }
    format!("{}...{}", &key[..4], &key[key.len() - 4..])
}
