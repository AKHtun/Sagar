//! Custom AI provider persistence. Secrets are stored in app-owned SQLite
//! and never returned to callers: reads produce masked summaries only. No
//! book content is involved; rows hold endpoint configuration.

use rusqlite::Connection;

use crate::domain::CustomProviderSummary;
use crate::error::AppError;

pub struct NewCustomProvider {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub created_at: String,
}

fn summary_from_row(row: &rusqlite::Row<'_>) -> Result<CustomProviderSummary, rusqlite::Error> {
    let api_key: String = row.get(3)?;
    Ok(CustomProviderSummary {
        id: row.get(0)?,
        name: row.get(1)?,
        base_url: row.get(2)?,
        has_api_key: !api_key.trim().is_empty(),
        model: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

/// Masked summaries in creation order (rowid), oldest first.
pub fn list_custom(conn: &Connection) -> Result<Vec<CustomProviderSummary>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, base_url, api_key, model, created_at, updated_at
         FROM ai_custom_providers ORDER BY rowid",
    )?;
    let rows = stmt.query_map([], summary_from_row)?;
    let mut providers = Vec::new();
    for row in rows {
        providers.push(row?);
    }
    Ok(providers)
}

pub fn create_custom(conn: &Connection, provider: &NewCustomProvider) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO ai_custom_providers
            (id, name, base_url, api_key, model, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        rusqlite::params![
            provider.id,
            provider.name,
            provider.base_url,
            provider.api_key,
            provider.model,
            provider.created_at,
        ],
    )?;
    Ok(())
}

/// Secret material for a connection test. Kept inside Rust; never serialized.
pub struct CustomSecret {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

pub fn get_secret(conn: &Connection, id: &str) -> Result<CustomSecret, AppError> {
    conn.query_row(
        "SELECT base_url, api_key, model FROM ai_custom_providers WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(CustomSecret {
                base_url: row.get(0)?,
                api_key: row.get(1)?,
                model: row.get(2)?,
            })
        },
    )
    .map_err(|_| {
        AppError::new(
            "provider_not_found",
            "This provider no longer exists in local settings.",
        )
    })
}

pub fn update_custom(
    conn: &Connection,
    id: &str,
    name: &str,
    base_url: &str,
    api_key: Option<&str>,
    model: &str,
    updated_at: &str,
) -> Result<CustomProviderSummary, AppError> {
    if api_key.is_some() {
        let changed = conn.execute(
            "UPDATE ai_custom_providers
             SET name = ?1, base_url = ?2, api_key = ?3, model = ?4, updated_at = ?5
             WHERE id = ?6",
            rusqlite::params![name, base_url, api_key, model, updated_at, id],
        )?;
        if changed == 0 {
            return Err(AppError::new(
                "provider_not_found",
                "This provider no longer exists in local settings.",
            ));
        }
    } else {
        let changed = conn.execute(
            "UPDATE ai_custom_providers
             SET name = ?1, base_url = ?2, model = ?3, updated_at = ?4
             WHERE id = ?5",
            rusqlite::params![name, base_url, model, updated_at, id],
        )?;
        if changed == 0 {
            return Err(AppError::new(
                "provider_not_found",
                "This provider no longer exists in local settings.",
            ));
        }
    }
    conn.query_row(
        "SELECT id, name, base_url, api_key, model, created_at, updated_at
         FROM ai_custom_providers WHERE id = ?1",
        rusqlite::params![id],
        summary_from_row,
    )
    .map_err(|_| {
        AppError::new(
            "provider_not_found",
            "This provider no longer exists in local settings.",
        )
    })
}

/// Returns false when the provider does not exist.
pub fn delete_custom(conn: &Connection, id: &str) -> Result<bool, AppError> {
    let deleted = conn.execute(
        "DELETE FROM ai_custom_providers WHERE id = ?1",
        rusqlite::params![id],
    )?;
    Ok(deleted > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::migrations;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().expect("open db");
        migrations::apply(&conn).expect("migrate");
        conn
    }

    fn sample(id: &str) -> NewCustomProvider {
        NewCustomProvider {
            id: id.into(),
            name: "Local AI".into(),
            base_url: "http://localhost:1234/v1".into(),
            api_key: "secret".into(),
            model: "local-model".into(),
            created_at: "2026-10-13T00:00:00Z".into(),
        }
    }

    #[test]
    fn summaries_mask_the_api_key() {
        let conn = setup();
        create_custom(&conn, &sample("p1")).expect("create");
        let list = list_custom(&conn).expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Local AI");
        assert!(list[0].has_api_key);
        // The raw key must not be recoverable from the summary path.
        let serialized = serde_json::to_string(&list[0]).expect("serialize");
        assert!(!serialized.contains("secret"), "{serialized}");
    }

    #[test]
    fn update_without_a_key_keeps_the_secret() {
        let conn = setup();
        create_custom(&conn, &sample("p1")).expect("create");
        update_custom(
            &conn,
            "p1",
            "Renamed",
            "http://localhost:1234/v1",
            None,
            "other-model",
            "2026-10-13T01:00:00Z",
        )
        .expect("update");
        let secret = get_secret(&conn, "p1").expect("secret");
        assert_eq!(secret.api_key, "secret");
        assert_eq!(secret.model, "other-model");
    }

    #[test]
    fn delete_reports_existence() {
        let conn = setup();
        assert!(!delete_custom(&conn, "ghost").expect("delete ghost"));
        create_custom(&conn, &sample("p1")).expect("create");
        assert!(delete_custom(&conn, "p1").expect("delete"));
        assert!(get_secret(&conn, "p1").is_err());
    }
}
