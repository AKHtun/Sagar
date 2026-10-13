//! AI provider settings (settings-only slice).
//!
//! Built-in CLI providers (Codex, OpenCode, Claude Code) are detected on
//! `PATH`; the list view never spawns processes or touches the network.
//! An explicit per-provider check runs `<binary> --version` with a timeout.
//! Custom OpenAI-compatible providers are stored in app-owned SQLite with
//! masked reads, and an explicit test performs a single `GET {base}/models`
//! request carrying only the stored key. No book content is ever sent.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::domain::{
    AiProviderList, CliCheckResult, CliProviderId, CliProviderStatus, CustomProviderSummary,
    CustomTestResult,
};
use crate::error::AppError;
use crate::storage::{self, ai_providers};

/// Bounded wait for `<binary> --version`.
const VERSION_TIMEOUT: Duration = Duration::from_secs(8);
/// Bounded wait for a custom provider `/models` probe.
const MODELS_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_VERSION_CHARS: usize = 120;

pub struct SaveCustomInput {
    pub id: Option<String>,
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
}

fn open_settings_db(data_dir: &Path) -> Result<rusqlite::Connection, AppError> {
    storage::initialize(data_dir)?;
    storage::open_database(&data_dir.join("sagar.db"))
}

// ------------------------------------------------------------ CLI lookup

/// File names tried for one binary. Windows installers (including npm shims)
/// use executable extensions; other systems use the bare name plus the
/// platform executable suffix.
fn candidate_file_names(binary: &str) -> Vec<OsString> {
    let mut names = vec![OsString::from(binary)];
    let exe_suffix = std::env::consts::EXE_SUFFIX;
    if !exe_suffix.is_empty() {
        names.push(OsString::from(format!("{binary}{exe_suffix}")));
    }
    if cfg!(windows) {
        for extension in [".cmd", ".bat"] {
            let candidate = format!("{binary}{extension}");
            if !names.iter().any(|name| name.to_string_lossy() == candidate) {
                names.push(OsString::from(candidate));
            }
        }
    }
    names
}

fn find_on_path_in(directories: &[PathBuf], binary: &str) -> Option<PathBuf> {
    let candidates = candidate_file_names(binary);
    for directory in directories {
        for candidate in &candidates {
            let path = directory.join(candidate);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

fn path_directories() -> Vec<PathBuf> {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).collect())
        .unwrap_or_default()
}

fn find_cli_binary(id: CliProviderId) -> Option<PathBuf> {
    let directories = path_directories();
    id.binaries()
        .iter()
        .find_map(|binary| find_on_path_in(&directories, binary))
}

fn cli_available(id: CliProviderId) -> bool {
    find_cli_binary(id).is_some()
}

/// Runs `<binary> --version` with a timeout. Only a fixed argument is ever
/// passed; no shell is involved. Errors are described generically so no
/// filesystem paths leak into messages or logs.
fn probe_cli_version(binary: &Path) -> Result<String, AppError> {
    let mut command = std::process::Command::new(binary);
    command
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = command.spawn().map_err(|_| {
        AppError::new(
            "cli_check_failed",
            "The tool was found but could not be started.",
        )
    })?;
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(child.wait_with_output());
    });
    let output = receiver
        .recv_timeout(VERSION_TIMEOUT)
        .map_err(|_| {
            AppError::new(
                "cli_check_timeout",
                "The tool was found but did not respond in time.",
            )
        })?
        .map_err(|_| {
            AppError::new(
                "cli_check_failed",
                "The tool was found but reported an error.",
            )
        })?;
    if !output.status.success() {
        return Err(AppError::new(
            "cli_check_failed",
            "The tool was found but reported an error.",
        ));
    }
    let text = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr)
    } else {
        String::from_utf8_lossy(&output.stdout)
    };
    let first_line = text.lines().next().unwrap_or("").trim();
    let version: String = first_line.chars().take(MAX_VERSION_CHARS).collect();
    if version.is_empty() {
        return Err(AppError::new(
            "cli_check_failed",
            "The tool responded without version information.",
        ));
    }
    Ok(version)
}

// ---------------------------------------------------------- validation

fn validate_name(raw: &str) -> Result<String, AppError> {
    let name = raw.trim().to_string();
    if name.is_empty() || name.chars().count() > 64 {
        return Err(AppError::new(
            "invalid_provider_name",
            "Enter a provider name of 1–64 characters.",
        ));
    }
    Ok(name)
}

fn validate_model(raw: &str) -> Result<String, AppError> {
    let model = raw.trim().to_string();
    if model.is_empty() || model.chars().count() > 128 {
        return Err(AppError::new(
            "invalid_model",
            "Enter a model name of 1–128 characters.",
        ));
    }
    Ok(model)
}

/// `required` on create; on update `None`/blank keeps the stored secret.
fn validate_api_key(raw: Option<&str>, required: bool) -> Result<Option<String>, AppError> {
    match raw.map(str::trim) {
        None | Some("") if !required => Ok(None),
        None | Some("") => Err(AppError::new("invalid_api_key", "Enter an API key.")),
        Some(key) => {
            if key.chars().count() > 2048 || key.contains(['\n', '\r']) {
                return Err(AppError::new(
                    "invalid_api_key",
                    "That API key looks invalid; check for extra line breaks.",
                ));
            }
            Ok(Some(key.to_string()))
        }
    }
}

/// Validates and normalizes an OpenAI-compatible base URL: explicit
/// http(s) scheme, no credentials, no query/fragment, no trailing slash.
pub fn normalize_base_url(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 2048 {
        return Err(AppError::new(
            "invalid_base_url",
            "Use a full http(s) URL such as https://api.example.com/v1.",
        ));
    }
    let mut url = reqwest::Url::parse(trimmed).map_err(|_| {
        AppError::new(
            "invalid_base_url",
            "Use a full http(s) URL such as https://api.example.com/v1.",
        )
    })?;
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(AppError::new(
            "invalid_base_url",
            "The URL must start with http:// or https://.",
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AppError::new(
            "invalid_base_url",
            "The URL must not contain credentials.",
        ));
    }
    if url.host_str().is_none_or(|host| host.is_empty()) {
        return Err(AppError::new(
            "invalid_base_url",
            "The URL must include a host name.",
        ));
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(AppError::new(
            "invalid_base_url",
            "The URL must not contain ? or # parts.",
        ));
    }
    let stripped = url.path().trim_end_matches('/').to_string();
    url.set_path(if stripped.is_empty() { "/" } else { &stripped });
    Ok(url.to_string().trim_end_matches('/').to_string())
}

// -------------------------------------------------------------- service

pub fn list_ai_providers(data_dir: &Path) -> Result<AiProviderList, AppError> {
    let conn = open_settings_db(data_dir)?;
    let cli = CliProviderId::all()
        .into_iter()
        .map(|id| CliProviderStatus {
            id: id.as_str().to_string(),
            label: id.label().to_string(),
            binaries: id.binaries().iter().map(|name| name.to_string()).collect(),
            available: cli_available(id),
        })
        .collect();
    let custom = ai_providers::list_custom(&conn)?;
    Ok(AiProviderList { cli, custom })
}

pub fn check_ai_cli(data_dir: &Path, provider_id: &str) -> Result<CliCheckResult, AppError> {
    let _ = open_settings_db(data_dir)?;
    let Some(id) = CliProviderId::parse(provider_id.trim()) else {
        return Err(AppError::new(
            "invalid_provider",
            "This provider is not supported.",
        ));
    };
    let Some(binary) = find_cli_binary(id) else {
        return Ok(CliCheckResult {
            id: id.as_str().to_string(),
            label: id.label().to_string(),
            available: false,
            connected: false,
            version: None,
            message: "Not found on PATH. Install the tool to use it here.".to_string(),
        });
    };
    match probe_cli_version(&binary) {
        Ok(version) => Ok(CliCheckResult {
            id: id.as_str().to_string(),
            label: id.label().to_string(),
            available: true,
            connected: true,
            version: Some(version),
            message: "The tool responded.".to_string(),
        }),
        Err(error) => Ok(CliCheckResult {
            id: id.as_str().to_string(),
            label: id.label().to_string(),
            available: true,
            connected: false,
            version: None,
            message: error.message,
        }),
    }
}

pub fn save_ai_custom_provider(
    data_dir: &Path,
    input: SaveCustomInput,
) -> Result<CustomProviderSummary, AppError> {
    let name = validate_name(&input.name)?;
    let base_url = normalize_base_url(&input.base_url)?;
    let model = validate_model(&input.model)?;
    let creating = input.id.as_deref().is_none_or(|id| id.trim().is_empty());
    let api_key = validate_api_key(input.api_key.as_deref(), creating)?;
    if !creating
        && let Some(id) = input.id.as_deref()
        && id.trim().is_empty()
    {
        return Err(AppError::new(
            "invalid_provider_id",
            "This provider cannot be saved safely.",
        ));
    }

    let conn = open_settings_db(data_dir)?;
    let now = chrono::Utc::now().to_rfc3339();
    if creating {
        let id = uuid::Uuid::new_v4().to_string();
        ai_providers::create_custom(
            &conn,
            &ai_providers::NewCustomProvider {
                id: id.clone(),
                name,
                base_url,
                api_key: api_key.unwrap_or_default(),
                model,
                created_at: now.clone(),
            },
        )?;
        let list = ai_providers::list_custom(&conn)?;
        list.into_iter()
            .find(|provider| provider.id == id)
            .ok_or_else(|| AppError::new("storage_error", "The provider could not be saved."))
    } else {
        let id = input.id.unwrap_or_default();
        ai_providers::update_custom(
            &conn,
            id.trim(),
            &name,
            &base_url,
            api_key.as_deref(),
            &model,
            &now,
        )
    }
}

pub fn delete_ai_custom_provider(data_dir: &Path, id: &str) -> Result<(), AppError> {
    if id.trim().is_empty() {
        return Err(AppError::new(
            "invalid_provider_id",
            "This provider cannot be removed safely.",
        ));
    }
    let conn = open_settings_db(data_dir)?;
    if !ai_providers::delete_custom(&conn, id.trim())? {
        return Err(AppError::new(
            "provider_not_found",
            "This provider no longer exists in local settings.",
        ));
    }
    Ok(())
}

/// User-triggered connectivity probe: a single `GET {base}/models` with the
/// stored key. No book content is attached to the request.
pub fn test_ai_custom_provider(data_dir: &Path, id: &str) -> Result<CustomTestResult, AppError> {
    if id.trim().is_empty() {
        return Err(AppError::new(
            "invalid_provider_id",
            "This provider cannot be tested safely.",
        ));
    }
    let conn = open_settings_db(data_dir)?;
    let secret = ai_providers::get_secret(&conn, id.trim())?;
    let base_url = normalize_base_url(&secret.base_url)?;
    if secret.api_key.trim().is_empty() {
        return Err(AppError::new(
            "invalid_api_key",
            "Add an API key before testing this provider.",
        ));
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(MODELS_TIMEOUT)
        .build()
        .map_err(|_| AppError::new("connection_failed", "The connection test could not start."))?;
    let response = client
        .get(format!("{base_url}/models"))
        .bearer_auth(secret.api_key)
        .header("Accept", "application/json")
        .send();
    let response = match response {
        Ok(response) => response,
        Err(error) if error.is_timeout() => {
            return Ok(CustomTestResult {
                ok: false,
                model_count: None,
                message: "The connection timed out.".to_string(),
            });
        }
        Err(error) if error.is_connect() => {
            return Ok(CustomTestResult {
                ok: false,
                model_count: None,
                message: "The server could not be reached.".to_string(),
            });
        }
        Err(_) => {
            return Ok(CustomTestResult {
                ok: false,
                model_count: None,
                message: "The connection test failed.".to_string(),
            });
        }
    };
    let status = response.status();
    if status.as_u16() == 401 || status.as_u16() == 403 {
        return Ok(CustomTestResult {
            ok: false,
            model_count: None,
            message: "The API key was rejected.".to_string(),
        });
    }
    if status.as_u16() == 404 {
        return Ok(CustomTestResult {
            ok: false,
            model_count: None,
            message: "No model list was found at this URL.".to_string(),
        });
    }
    if !status.is_success() {
        return Ok(CustomTestResult {
            ok: false,
            model_count: None,
            message: format!("The server returned an error (status {}).", status.as_u16()),
        });
    }
    let model_count = response
        .json::<serde_json::Value>()
        .ok()
        .and_then(|body| body.get("data")?.as_array().map(|items| items.len()));
    Ok(CustomTestResult {
        ok: true,
        model_count,
        message: match model_count {
            Some(count) => format!("Connected — {count} models listed."),
            None => "Connected.".to_string(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_url_normalization_keeps_the_path_and_rejects_risky_urls() {
        assert_eq!(
            normalize_base_url("https://api.example.com/v1/").expect("trailing slash"),
            "https://api.example.com/v1"
        );
        assert_eq!(
            normalize_base_url("http://localhost:1234/v1").expect("local http"),
            "http://localhost:1234/v1"
        );
        for rejected in [
            "api.example.com/v1",
            "ftp://example.com/v1",
            "https://user:pass@example.com/v1",
            "https://example.com/v1?key=secret",
            "https://example.com/v1#models",
            "https://",
            "",
        ] {
            assert!(
                normalize_base_url(rejected).is_err(),
                "must reject {rejected}"
            );
        }
    }

    #[test]
    fn provider_field_limits_are_enforced() {
        assert!(validate_name("  ").is_err());
        assert!(validate_name(&"n".repeat(65)).is_err());
        assert!(validate_model("").is_err());
        assert!(validate_api_key(None, true).is_err());
        assert!(
            validate_api_key(Some(""), false)
                .expect("blank keeps")
                .is_none()
        );
        assert!(validate_api_key(Some("line\nbreak"), true).is_err());
    }

    #[test]
    fn path_lookup_finds_staged_binaries_without_a_shell() {
        let dir = tempfile::tempdir().expect("temp dir");
        let binary = dir.path().join("codex");
        std::fs::write(&binary, b"stub").expect("stage binary");
        let found = find_on_path_in(&[dir.path().to_path_buf()], "codex").expect("binary found");
        assert_eq!(found, binary);
        assert!(find_on_path_in(&[dir.path().to_path_buf()], "ghost").is_none());
        assert!(find_on_path_in(&[], "codex").is_none());
    }

    #[test]
    fn saving_rejects_invalid_input_before_storage() {
        let dir = tempfile::tempdir().expect("temp dir");
        let error = save_ai_custom_provider(
            dir.path(),
            SaveCustomInput {
                id: None,
                name: "Local".into(),
                base_url: "not-a-url".into(),
                api_key: Some("key".into()),
                model: "m".into(),
            },
        )
        .expect_err("bad URL rejected");
        assert_eq!(error.code, "invalid_base_url");

        let created = save_ai_custom_provider(
            dir.path(),
            SaveCustomInput {
                id: None,
                name: "Local".into(),
                base_url: "http://localhost:1234/v1/".into(),
                api_key: Some("key".into()),
                model: "m".into(),
            },
        )
        .expect("create");
        assert_eq!(created.base_url, "http://localhost:1234/v1");
        assert!(created.has_api_key);
        let serialized = serde_json::to_string(&created).expect("serialize");
        assert!(!serialized.contains("key"), "{serialized}");
    }
}
