//! Translation persistence: block views, optimistic-revision saves and
//! chapter status derivation. All SQL for editing lives here.

use rusqlite::Connection;

use crate::domain::{BlockView, ChapterContent, ChapterReview, SaveResult};
use crate::error::AppError;

/// Loads a chapter with every block's translation state.
pub fn get_chapter_content(
    conn: &Connection,
    chapter_id: &str,
    sanitize_fragment: &dyn Fn(&str) -> String,
) -> Result<ChapterContent, AppError> {
    let chapter = conn
        .query_row(
            "SELECT title, resource_href FROM chapters WHERE id = ?1",
            rusqlite::params![chapter_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|_| {
            AppError::new(
                "chapter_not_found",
                "This chapter no longer exists in local storage.",
            )
        })?;

    let mut stmt = conn.prepare(
        "SELECT b.id, b.block_index, b.element_name, b.structural_locator,
                b.source_xhtml_fragment, b.source_plain_text, b.support_status,
                t.target_plain_text, t.status, t.revision, t.target_xhtml_fragment
         FROM blocks b
         JOIN translations t ON t.block_id = b.id
         WHERE b.chapter_id = ?1
         ORDER BY b.block_index",
    )?;
    let rows = stmt.query_map(rusqlite::params![chapter_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, i64>(9)?,
            row.get::<_, Option<String>>(10)?,
        ))
    })?;

    let mut blocks = Vec::new();
    for row in rows {
        let (
            id,
            block_index,
            element_name,
            structural_locator,
            fragment,
            plain_text,
            support_status,
            target_plain_text,
            status,
            revision,
            target_html,
        ) = row?;
        blocks.push(BlockView {
            id,
            block_index,
            element_name,
            structural_locator,
            source_html: sanitize_fragment(&fragment),
            has_inline_markup: fragment.contains('<'),
            source_plain_text: plain_text,
            source_editor_html: crate::epub::rich_text::validate(&fragment)
                .ok()
                .map(|(html, _)| html),
            support_status,
            target_plain_text,
            target_html,
            status,
            revision,
        });
    }

    Ok(ChapterContent {
        chapter_id: chapter_id.to_string(),
        title: chapter.0,
        resource_href: chapter.1,
        blocks,
        source_document: String::new(),
        preview_html: String::new(),
        images: Vec::new(),
        document: crate::storage::documents::load(conn, chapter_id)?,
    })
}

/// Saves a translation with optimistic revision checking.
///
/// `target_text` is plain, trusted-to-be-plain text from the restricted
/// editor; the fragment stored for export is escaped here so no markup can
/// enter the pipeline. Review status changes follow the design rules:
/// reviewed requires content, skipped keeps the original.
pub struct SaveInput {
    pub block_id: String,
    pub target_text: Option<String>,
    pub status: String,
    pub expected_revision: i64,
}

pub fn save_translation(conn: &Connection, input: SaveInput) -> Result<SaveResult, AppError> {
    let tx = conn.unchecked_transaction()?;
    let result = save_inner(&tx, input)?;
    tx.commit()?;
    Ok(result)
}

fn save_inner(conn: &Connection, input: SaveInput) -> Result<SaveResult, AppError> {
    let now = chrono::Utc::now().to_rfc3339();

    let current = conn
        .query_row(
            "SELECT t.revision, b.chapter_id, b.support_status
             FROM translations t JOIN blocks b ON b.id = t.block_id
             WHERE t.block_id = ?1",
            rusqlite::params![input.block_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .map_err(|_| {
            AppError::new(
                "block_not_found",
                "This block no longer exists in local storage.",
            )
        })?;
    let (current_revision, chapter_id, support_status) = current;

    if current_revision != input.expected_revision {
        return Err(AppError::with_details(
            "revision_conflict",
            "A newer saved version of this block exists. Reload the chapter \
             to continue from the saved content.",
            serde_json::json!({ "currentRevision": current_revision }),
        ));
    }
    if support_status != "supported" {
        return Err(AppError::new(
            "block_unsupported",
            "This block is preserved in the original language and cannot be \
             edited.",
        ));
    }

    let trimmed = input.target_text.as_deref().map(str::trim).unwrap_or("");
    let has_content = !trimmed.is_empty();
    match input.status.as_str() {
        "reviewed" if !has_content => {
            return Err(AppError::new(
                "review_requires_content",
                "Empty blocks cannot be marked reviewed. Use “Keep original” \
                 instead.",
            ));
        }
        "pending" | "draft" | "reviewed" | "skipped" => {}
        other => {
            return Err(AppError::new(
                "invalid_status",
                format!("“{other}” is not a valid block status."),
            ));
        }
    }

    // Skipped blocks keep the original: no target content is stored.
    let (target_text, target_fragment) = if input.status == "skipped" || !has_content {
        (None, None)
    } else {
        let text = trimmed.to_string();
        let fragment = escape_inner_text(&text);
        (Some(text), Some(fragment))
    };
    // Content-less saves fall back to pending unless the user explicitly
    // chose to keep the original (skipped is content-less by design).
    let status = if has_content || input.status == "skipped" {
        input.status.as_str()
    } else {
        "pending"
    };
    let reviewed_at = if status == "reviewed" {
        Some(now.clone())
    } else {
        None
    };

    let new_revision = current_revision + 1;
    conn.execute(
        "UPDATE translations
         SET target_xhtml_fragment = ?1, target_plain_text = ?2, status = ?3,
             revision = ?4, updated_at = ?5, reviewed_at = ?6
         WHERE block_id = ?7 AND revision = ?8",
        rusqlite::params![
            target_fragment,
            target_text,
            status,
            new_revision,
            now,
            reviewed_at,
            input.block_id,
            current_revision,
        ],
    )?;
    // (execute returning 0 rows would mean a raced write; the revision
    // check above plus the WHERE revision clause guards this.)

    let chapter_status = refresh_chapter_status(conn, &chapter_id)?;
    conn.execute(
        "UPDATE projects SET updated_at = ?1
         WHERE id = (SELECT project_id FROM chapters WHERE id = ?2)",
        rusqlite::params![now, chapter_id],
    )?;

    Ok(SaveResult {
        block_id: input.block_id,
        revision: new_revision,
        status: status.to_string(),
        saved_at: now,
        chapter_status,
    })
}

/// Recomputes the chapter status from its block states and stores it.
pub fn refresh_chapter_status(conn: &Connection, chapter_id: &str) -> Result<String, AppError> {
    let (supported, reviewed_or_skipped, started): (i64, i64, i64) = conn.query_row(
        "SELECT COUNT(*),
                COALESCE(SUM(CASE WHEN t.status IN ('reviewed', 'skipped') THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN t.status != 'pending' OR t.target_plain_text IS NOT NULL
                                  THEN 1 ELSE 0 END), 0)
         FROM blocks b JOIN translations t ON t.block_id = b.id
         WHERE b.chapter_id = ?1 AND b.support_status = 'supported'",
        rusqlite::params![chapter_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let status = if supported > 0 && reviewed_or_skipped == supported {
        "completed"
    } else if started > 0 {
        "in_progress"
    } else {
        "not_started"
    };
    conn.execute(
        "UPDATE chapters SET status = ?1 WHERE id = ?2",
        rusqlite::params![status, chapter_id],
    )?;
    Ok(status.to_string())
}

/// Records the chapter the user is working in, for restart resume.
pub fn set_last_chapter(
    conn: &Connection,
    project_id: &str,
    chapter_id: &str,
) -> Result<(), AppError> {
    conn.execute(
        "UPDATE projects SET last_chapter_id = ?1 WHERE id = ?2",
        rusqlite::params![chapter_id, project_id],
    )?;
    Ok(())
}

pub fn get_last_chapter(conn: &Connection, project_id: &str) -> Result<Option<String>, AppError> {
    let value: Option<String> = conn.query_row(
        "SELECT last_chapter_id FROM projects WHERE id = ?1",
        rusqlite::params![project_id],
        |row| row.get(0),
    )?;
    Ok(value.filter(|id| {
        conn.query_row(
            "SELECT COUNT(*) FROM chapters WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false)
    }))
}

/// Builds the export review report from current translation state.
pub fn export_review(
    conn: &Connection,
    project_id: &str,
) -> Result<crate::domain::ExportReview, AppError> {
    let project = conn
        .query_row(
            "SELECT title, source_language, target_language, translated_title
         FROM projects WHERE id = ?1",
            rusqlite::params![project_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .map_err(|_| {
            AppError::new(
                "project_not_found",
                "This project no longer exists in local storage.",
            )
        })?;

    let mut stmt = conn.prepare(
        "SELECT ch.id, ch.spine_index, ch.title,
                COUNT(b.id),
                COALESCE(SUM(t.status = 'reviewed'), 0),
                COALESCE(SUM(t.status = 'skipped'), 0),
                COALESCE(SUM(t.status = 'draft'), 0),
                COALESCE(SUM(t.status = 'pending'), 0),
                COALESCE(SUM(b.support_status != 'supported'), 0)
         FROM chapters ch
         JOIN blocks b ON b.chapter_id = ch.id
         JOIN translations t ON t.block_id = b.id
         WHERE ch.project_id = ?1
         GROUP BY ch.id
         ORDER BY ch.spine_index",
    )?;
    let mut chapters = Vec::new();
    let (
        mut reviewed_total,
        mut skipped_total,
        mut draft_total,
        mut pending_total,
        mut unsupported_total,
    ) = (0, 0, 0, 0, 0);
    let rows = stmt.query_map(rusqlite::params![project_id], |row| {
        Ok(ChapterReview {
            chapter_id: row.get(0)?,
            spine_index: row.get(1)?,
            title: row.get(2)?,
            total_blocks: row.get(3)?,
            reviewed: row.get(4)?,
            skipped: row.get(5)?,
            draft: row.get(6)?,
            pending: row.get(7)?,
            unsupported: row.get(8)?,
        })
    })?;
    for row in rows {
        let review = row?;
        reviewed_total += review.reviewed;
        skipped_total += review.skipped;
        draft_total += review.draft;
        pending_total += review.pending;
        unsupported_total += review.unsupported;
        chapters.push(review);
    }

    Ok(crate::domain::ExportReview {
        project_id: project_id.to_string(),
        title: project.0,
        source_language: project.1,
        target_language: project.2,
        translated_title: project.3,
        complete: pending_total == 0 && draft_total == 0 && !chapters.is_empty(),
        chapters,
        reviewed_total,
        skipped_total,
        draft_total,
        pending_total,
        unsupported_total,
    })
}

/// Per-block translation rows needed for export patching.
pub struct BlockForExport {
    pub structural_locator: String,
    pub element_name: String,
    pub target_fragment: Option<String>,
    pub status: String,
}

pub fn blocks_for_export(
    conn: &Connection,
    chapter_id: &str,
) -> Result<Vec<BlockForExport>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT b.structural_locator, b.element_name, t.target_xhtml_fragment, t.status
         FROM blocks b JOIN translations t ON t.block_id = b.id
         WHERE b.chapter_id = ?1
         ORDER BY b.block_index",
    )?;
    let rows = stmt.query_map(rusqlite::params![chapter_id], |row| {
        Ok(BlockForExport {
            structural_locator: row.get(0)?,
            element_name: row.get(1)?,
            target_fragment: row.get(2)?,
            status: row.get(3)?,
        })
    })?;
    let mut blocks = Vec::new();
    for row in rows {
        blocks.push(row?);
    }
    Ok(blocks)
}

/// Escapes text as inner XHTML content (the only markup the restricted
/// editor can produce is none).
pub fn escape_inner_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::migrations;
    use crate::storage::projects::{self, NewBlock, NewChapter, NewProject};

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().expect("open db");
        migrations::apply(&conn).expect("migrate");
        projects::create_project(
            &conn,
            &NewProject {
                original_source_path: None,
                id: "p1".into(),
                source_sha256: "abc".into(),
                source_snapshot_path: "projects/p1/source.epub".into(),
                title: "T".into(),
                author: None,
                source_language: Some("en".into()),
                created_at: "2026-01-01T00:00:00Z".into(),
            },
            &[(
                NewChapter {
                    id: "c1".into(),
                    spine_index: 0,
                    manifest_item_id: "m1".into(),
                    resource_href: "c1.xhtml".into(),
                    title: "Chapter".into(),
                },
                vec![
                    NewBlock {
                        id: "b1".into(),
                        block_index: 0,
                        structural_locator: "0".into(),
                        element_name: "p".into(),
                        source_xhtml_fragment: "One".into(),
                        source_plain_text: "One".into(),
                        is_translatable: true,
                        supported: true,
                    },
                    NewBlock {
                        id: "b2".into(),
                        block_index: 1,
                        structural_locator: "1".into(),
                        element_name: "p".into(),
                        source_xhtml_fragment: "Two".into(),
                        source_plain_text: "Two".into(),
                        is_translatable: true,
                        supported: true,
                    },
                ],
            )],
        )
        .expect("create project");
        conn
    }

    fn identity(fragment: &str) -> String {
        fragment.to_string()
    }

    #[test]
    fn save_updates_revision_status_and_chapter_completion() {
        let conn = setup();

        let result = save_translation(
            &conn,
            SaveInput {
                block_id: "b1".into(),
                target_text: Some("တစ်".into()),
                status: "draft".into(),
                expected_revision: 0,
            },
        )
        .expect("save draft");
        assert_eq!(result.revision, 1);
        assert_eq!(result.status, "draft");
        assert_eq!(result.chapter_status, "in_progress");

        let result = save_translation(
            &conn,
            SaveInput {
                block_id: "b1".into(),
                target_text: Some("တစ်".into()),
                status: "reviewed".into(),
                expected_revision: 1,
            },
        )
        .expect("mark reviewed");
        assert_eq!(result.status, "reviewed");

        let content = get_chapter_content(&conn, "c1", &identity).expect("content");
        assert_eq!(content.blocks[0].revision, 2);
        assert_eq!(content.blocks[0].status, "reviewed");
        assert_eq!(content.blocks[0].target_plain_text.as_deref(), Some("တစ်"));

        // Second block reviewed → chapter completed.
        save_translation(
            &conn,
            SaveInput {
                block_id: "b2".into(),
                target_text: Some("နှစ်".into()),
                status: "reviewed".into(),
                expected_revision: 0,
            },
        )
        .expect("save b2");
        let content = get_chapter_content(&conn, "c1", &identity).expect("content");
        assert_eq!(content.blocks[0].status, "reviewed");
        let status: String = conn
            .query_row("SELECT status FROM chapters WHERE id='c1'", [], |r| {
                r.get(0)
            })
            .expect("chapter status");
        assert_eq!(status, "completed");
    }

    #[test]
    fn stale_revision_is_rejected_with_conflict() {
        let conn = setup();
        save_translation(
            &conn,
            SaveInput {
                block_id: "b1".into(),
                target_text: Some("first".into()),
                status: "draft".into(),
                expected_revision: 0,
            },
        )
        .expect("first save");
        let error = save_translation(
            &conn,
            SaveInput {
                block_id: "b1".into(),
                target_text: Some("second".into()),
                status: "draft".into(),
                expected_revision: 0,
            },
        )
        .expect_err("stale save must conflict");
        assert_eq!(error.code, "revision_conflict");
    }

    #[test]
    fn reviewed_requires_content_and_skipped_stores_none() {
        let conn = setup();
        let error = save_translation(
            &conn,
            SaveInput {
                block_id: "b1".into(),
                target_text: Some("  ".into()),
                status: "reviewed".into(),
                expected_revision: 0,
            },
        )
        .expect_err("reviewed without content");
        assert_eq!(error.code, "review_requires_content");

        let result = save_translation(
            &conn,
            SaveInput {
                block_id: "b1".into(),
                target_text: None,
                status: "skipped".into(),
                expected_revision: 0,
            },
        )
        .expect("skip");
        assert_eq!(result.status, "skipped");
        let content = get_chapter_content(&conn, "c1", &identity).expect("content");
        assert_eq!(content.blocks[0].target_plain_text, None);
        assert_eq!(content.blocks[0].status, "skipped");
    }

    #[test]
    fn markup_in_translation_text_is_escaped() {
        let conn = setup();
        save_translation(
            &conn,
            SaveInput {
                block_id: "b1".into(),
                target_text: Some("a <b> & c".into()),
                status: "draft".into(),
                expected_revision: 0,
            },
        )
        .expect("save");
        let fragment: String = conn
            .query_row(
                "SELECT target_xhtml_fragment FROM translations WHERE block_id='b1'",
                [],
                |r| r.get(0),
            )
            .expect("fragment");
        assert_eq!(fragment, "a &lt;b&gt; &amp; c");
    }

    #[test]
    fn last_chapter_round_trips_and_validates() {
        let conn = setup();
        set_last_chapter(&conn, "p1", "c1").expect("set");
        assert_eq!(
            get_last_chapter(&conn, "p1").expect("get"),
            Some("c1".to_string())
        );
        set_last_chapter(&conn, "p1", "ghost").expect("set ghost");
        assert_eq!(get_last_chapter(&conn, "p1").expect("get"), None);
    }

    #[test]
    fn export_review_counts_reflect_saves() {
        let conn = setup();
        save_translation(
            &conn,
            SaveInput {
                block_id: "b1".into(),
                target_text: Some("done".into()),
                status: "reviewed".into(),
                expected_revision: 0,
            },
        )
        .expect("save");
        let review = export_review(&conn, "p1").expect("review");
        assert_eq!(review.reviewed_total, 1);
        assert_eq!(review.pending_total, 1);
        assert!(!review.complete);
    }
}

/// Rich text uses the same revision service, in one transaction.
pub fn save_rich_translation(
    conn: &Connection,
    block_id: String,
    fragment: Option<String>,
    status: String,
    expected_revision: i64,
) -> Result<SaveResult, AppError> {
    let (html, plain) = crate::epub::rich_text::validate(fragment.as_deref().unwrap_or(""))?;
    if status != "skipped" && !plain.trim().is_empty() {
        let source: String = conn.query_row(
            "SELECT source_xhtml_fragment FROM blocks WHERE id=?1",
            rusqlite::params![block_id],
            |row| row.get(0),
        )?;
        let wrapped = format!(
            "<w xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\">{source}</w>"
        );
        if let Ok(doc) = roxmltree::Document::parse(&wrapped) {
            let target_wrapped = format!("<w>{html}</w>");
            let target_doc = roxmltree::Document::parse(&target_wrapped)
                .map_err(|_| AppError::new("invalid_translation", "Invalid translation markup."))?;
            for node in doc.descendants().filter(|n| n.is_element()) {
                if node.attribute("id").is_some()
                    || node.has_tag_name(("http://www.w3.org/1999/xhtml", "ruby"))
                {
                    return Err(AppError::new(
                        "unsupported_inline_content",
                        "This passage contains an inline anchor or ruby annotation that cannot be safely edited yet. Choose Keep original.",
                    ));
                }
                if node.has_tag_name(("http://www.w3.org/1999/xhtml", "a")) {
                    if let Some(href) = node.attribute("href") {
                        if !target_doc
                            .descendants()
                            .any(|n| n.has_tag_name("a") && n.attribute("href") == Some(href))
                        {
                            return Err(AppError::new(
                                "missing_source_link",
                                "Preserve this passage’s original link: use Copy original and edit the linked text, or choose Keep original.",
                            ));
                        }
                    }
                }
            }
        }
    }
    let tx = conn.unchecked_transaction()?;
    let conn = &tx;
    let result: Result<SaveResult, AppError> = (|| {
        let result = save_inner(
            conn,
            SaveInput {
                block_id: block_id.clone(),
                target_text: Some(plain),
                status,
                expected_revision,
            },
        )?;
        if !matches!(result.status.as_str(), "skipped" | "pending") {
            conn.execute(
                "UPDATE translations SET target_xhtml_fragment=?1 WHERE block_id=?2",
                rusqlite::params![html, block_id],
            )?;
        }
        Ok(result)
    })();
    let result = result?;
    tx.commit()?;
    Ok(result)
}
