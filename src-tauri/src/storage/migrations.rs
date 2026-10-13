//! Ordered, embedded SQLite migrations.
//!
//! Each entry bumps `PRAGMA user_version` to its version inside a
//! transaction. Migrations must never mutate data destructively without an
//! explicit migration step; never drop a user's translation database.

use crate::error::AppError;
use rusqlite::Connection;

/// (version, SQL script). Versions must be strictly increasing.
const MIGRATIONS: &[(i64, &str)] = &[
    (
        1,
        "CREATE TABLE IF NOT EXISTS app_meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    ),
    // Translation project schema from dev/plan.md §5 (proposed SQLite v1).
    (
        2,
        "CREATE TABLE projects (
            id                   TEXT PRIMARY KEY,
            source_sha256        TEXT NOT NULL,
            source_snapshot_path TEXT NOT NULL,
            title                TEXT NOT NULL,
            author               TEXT,
            source_language      TEXT,
            target_language      TEXT NOT NULL DEFAULT 'my',
            translated_title     TEXT,
            created_at           TEXT NOT NULL,
            updated_at           TEXT NOT NULL,
            last_chapter_id      TEXT,
            schema_version       INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE chapters (
            id               TEXT PRIMARY KEY,
            project_id       TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            spine_index      INTEGER NOT NULL,
            manifest_item_id TEXT NOT NULL,
            resource_href    TEXT NOT NULL,
            title            TEXT NOT NULL,
            status           TEXT NOT NULL DEFAULT 'not_started',
            UNIQUE (project_id, spine_index)
        );

        CREATE TABLE blocks (
            id                    TEXT PRIMARY KEY,
            chapter_id            TEXT NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
            block_index           INTEGER NOT NULL,
            structural_locator    TEXT NOT NULL,
            element_name          TEXT NOT NULL,
            source_xhtml_fragment TEXT NOT NULL,
            source_plain_text     TEXT NOT NULL,
            is_translatable       INTEGER NOT NULL DEFAULT 1,
            support_status        TEXT NOT NULL DEFAULT 'supported',
            UNIQUE (chapter_id, block_index)
        );

        CREATE TABLE translations (
            block_id              TEXT PRIMARY KEY REFERENCES blocks(id) ON DELETE CASCADE,
            target_xhtml_fragment TEXT,
            target_plain_text     TEXT,
            status                TEXT NOT NULL DEFAULT 'pending',
            revision              INTEGER NOT NULL DEFAULT 0,
            updated_at            TEXT,
            reviewed_at           TEXT
        );

        CREATE INDEX idx_chapters_project ON chapters(project_id, spine_index);
        CREATE INDEX idx_blocks_chapter ON blocks(chapter_id, block_index);

        CREATE VIEW project_progress AS
        SELECT ch.project_id AS project_id,
               COUNT(b.id) AS total,
               SUM(CASE WHEN t.status IN ('reviewed', 'skipped')
                        THEN 1 ELSE 0 END) AS reviewed,
               SUM(CASE WHEN b.support_status != 'supported'
                        THEN 1 ELSE 0 END) AS unsupported
        FROM chapters ch
        JOIN blocks b ON b.chapter_id = ch.id
        LEFT JOIN translations t ON t.block_id = b.id
        GROUP BY ch.project_id;",
    ),
    (
        3,
        "CREATE TABLE chapter_documents (
        chapter_id TEXT PRIMARY KEY REFERENCES chapters(id) ON DELETE CASCADE,
        html TEXT,
        status TEXT NOT NULL DEFAULT 'pending',
        revision INTEGER NOT NULL DEFAULT 0,
        authored INTEGER NOT NULL DEFAULT 0,
        updated_at TEXT
    );
    DROP VIEW project_progress;
    CREATE VIEW project_progress AS
    SELECT ch.project_id, COUNT(ch.id) AS total,
        COALESCE(SUM(d.status IN ('reviewed','skipped')),0) AS reviewed,
        0 AS unsupported
    FROM chapters ch LEFT JOIN chapter_documents d ON d.chapter_id=ch.id GROUP BY ch.project_id;",
    ),
    // AI provider settings (settings-only slice): user-added OpenAI-compatible
    // endpoints. Secrets stay in app-owned SQLite and are never serialized
    // back; summaries only report key presence.
    (
        4,
        "CREATE TABLE ai_custom_providers (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            base_url    TEXT NOT NULL,
            api_key     TEXT NOT NULL,
            model       TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );",
    ),
];

/// Highest defined schema version. Currently asserted on by tests; production
/// callers (import, export) start using it from Milestone 1.
#[cfg(test)]
pub fn latest_version() -> i64 {
    MIGRATIONS
        .iter()
        .map(|(version, _)| *version)
        .max()
        .unwrap_or(0)
}

/// Applies pending migrations and returns the resulting schema version.
/// Safe to call repeatedly: already-applied scripts are skipped.
pub fn apply(conn: &Connection) -> Result<i64, AppError> {
    let mut version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    for (target, script) in MIGRATIONS {
        let target = *target;
        if target <= version {
            continue;
        }
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(script)?;
        tx.pragma_update(None, "user_version", target)?;
        tx.commit()?;
        version = target;
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_ordered_with_unique_versions() {
        let mut previous = 0;
        for (version, _) in MIGRATIONS {
            assert!(
                *version > previous,
                "migration versions must strictly increase"
            );
            previous = *version;
        }
    }
}
