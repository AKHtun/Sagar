//! Thin Tauri command adapters. Business logic lives in `services`; commands
//! only translate between the WebView and service calls.

use tauri::{AppHandle, Manager};

use crate::domain::{
    ChapterContent, ChapterSummary, CoverImage, ExportOutcome, ExportReview, ImportResult,
    ProjectSummary, SaveResult,
};
use crate::error::AppError;
use crate::services::{self, HealthReport};
use crate::storage::translations::SaveInput;

#[tauri::command]
pub fn health_check(app: AppHandle) -> Result<HealthReport, AppError> {
    services::health_check(&app)
}

fn app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    app.path()
        .app_data_dir()
        .map_err(|error| AppError::new("path_error", error.to_string()))
}

/// Long-running file work stays off the UI thread.
async fn run_blocking<T, F>(task: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|error| AppError::new("task_failed", error.to_string()))?
}

#[tauri::command]
pub async fn import_epub(
    app: AppHandle,
    source_path: String,
    allow_duplicate: bool,
) -> Result<ImportResult, AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || {
        services::import::import_epub(
            &data_dir,
            std::path::Path::new(&source_path),
            allow_duplicate,
        )
    })
    .await
}

#[tauri::command]
pub async fn list_projects(app: AppHandle) -> Result<Vec<ProjectSummary>, AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || services::projects::list_projects(&data_dir)).await
}

#[tauri::command]
pub async fn get_project(app: AppHandle, project_id: String) -> Result<ProjectSummary, AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || services::projects::get_project(&data_dir, &project_id)).await
}

#[tauri::command]
pub async fn list_chapters(
    app: AppHandle,
    project_id: String,
) -> Result<Vec<ChapterSummary>, AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || services::projects::list_chapters(&data_dir, &project_id)).await
}

#[tauri::command]
pub async fn delete_project(app: AppHandle, project_id: String) -> Result<(), AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || services::projects::delete_project(&data_dir, &project_id)).await
}

#[tauri::command]
pub async fn get_project_cover(
    app: AppHandle,
    project_id: String,
) -> Result<Option<CoverImage>, AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || services::projects::get_cover(&data_dir, &project_id)).await
}

#[tauri::command]
pub async fn get_chapter(
    app: AppHandle,
    project_id: String,
    chapter_id: String,
) -> Result<ChapterContent, AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || services::workspace::get_chapter(&data_dir, &project_id, &chapter_id))
        .await
}

#[tauri::command]
pub async fn save_translation(
    app: AppHandle,
    block_id: String,
    target_text: Option<String>,
    status: String,
    expected_revision: i64,
) -> Result<SaveResult, AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || {
        services::workspace::save_translation(
            &data_dir,
            SaveInput {
                block_id,
                target_text,
                status,
                expected_revision,
            },
        )
    })
    .await
}

#[tauri::command]
pub async fn get_last_chapter(
    app: AppHandle,
    project_id: String,
) -> Result<Option<String>, AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || services::workspace::get_last_chapter(&data_dir, &project_id)).await
}

#[tauri::command]
pub async fn update_project_metadata(
    app: AppHandle,
    project_id: String,
    translated_title: Option<String>,
    target_language: Option<String>,
) -> Result<(), AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || {
        services::workspace::update_project_metadata(
            &data_dir,
            &project_id,
            translated_title,
            target_language,
        )
    })
    .await
}

#[tauri::command]
pub async fn review_export(app: AppHandle, project_id: String) -> Result<ExportReview, AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || services::export::review_export(&data_dir, &project_id)).await
}

#[tauri::command]
pub async fn export_epub(
    app: AppHandle,
    project_id: String,
    destination: String,
    mode: String,
    title: Option<String>,
    language: Option<String>,
    include_original: Option<bool>,
) -> Result<ExportOutcome, AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || {
        services::export::export_epub(
            &data_dir,
            services::export::ExportRequest {
                project_id,
                destination: std::path::PathBuf::from(destination),
                mode,
                title,
                language,
                include_original: include_original.unwrap_or(true),
            },
        )
    })
    .await
}

#[tauri::command]
pub async fn save_rich_translation(
    app: AppHandle,
    block_id: String,
    target_html: Option<String>,
    status: String,
    expected_revision: i64,
) -> Result<SaveResult, AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || {
        services::workspace::save_rich_translation(
            &data_dir,
            block_id,
            target_html,
            status,
            expected_revision,
        )
    })
    .await
}

#[tauri::command]
pub async fn save_chapter_document(
    app: AppHandle,
    chapter_id: String,
    html: Option<String>,
    status: String,
    expected_revision: i64,
) -> Result<SaveResult, AppError> {
    let data_dir = app_data_dir(&app)?;
    run_blocking(move || {
        services::workspace::save_chapter_document(
            &data_dir,
            &chapter_id,
            html,
            &status,
            expected_revision,
        )
    })
    .await
}

#[tauri::command]
pub async fn validate_chapter_html(html: String) -> Result<String, AppError> {
    run_blocking(move || crate::epub::rich_text::validate_document(&html).map(|(html, _)| html))
        .await
}
