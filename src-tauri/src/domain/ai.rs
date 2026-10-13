//! AI provider settings models (settings-only slice).
//!
//! No book content is ever attached to these DTOs. API keys are never
//! serialized back to the frontend; summaries only report whether a key is
//! present.

/// Built-in CLI provider identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliProviderId {
    Codex,
    Opencode,
    ClaudeCode,
}

impl CliProviderId {
    pub fn as_str(&self) -> &'static str {
        match self {
            CliProviderId::Codex => "codex",
            CliProviderId::Opencode => "opencode",
            CliProviderId::ClaudeCode => "claude_code",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            CliProviderId::Codex => "Codex",
            CliProviderId::Opencode => "OpenCode",
            CliProviderId::ClaudeCode => "Claude Code",
        }
    }

    /// Binary names searched on `PATH` (without OS suffixes; Windows
    /// extensions are added at lookup time).
    pub fn binaries(&self) -> &'static [&'static str] {
        match self {
            CliProviderId::Codex => &["codex"],
            CliProviderId::Opencode => &["opencode"],
            CliProviderId::ClaudeCode => &["claude"],
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "codex" => Some(CliProviderId::Codex),
            "opencode" => Some(CliProviderId::Opencode),
            "claude_code" => Some(CliProviderId::ClaudeCode),
            _ => None,
        }
    }

    pub fn all() -> [CliProviderId; 3] {
        [
            CliProviderId::Codex,
            CliProviderId::Opencode,
            CliProviderId::ClaudeCode,
        ]
    }
}

/// PATH-only availability for one built-in CLI (no process is spawned and no
/// network request is made for the list view).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliProviderStatus {
    pub id: String,
    pub label: String,
    pub binaries: Vec<String>,
    pub available: bool,
}

/// Result of an explicit per-provider check: PATH lookup plus a bounded
/// `<binary> --version` probe. No book content or API keys are involved.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliCheckResult {
    pub id: String,
    pub label: String,
    pub available: bool,
    pub connected: bool,
    pub version: Option<String>,
    /// Short human-readable outcome; never contains filesystem paths.
    pub message: String,
}

/// A user-added OpenAI-compatible provider. The secret itself is never
/// serialized; only its presence is reported.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomProviderSummary {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub has_api_key: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderList {
    pub cli: Vec<CliProviderStatus>,
    pub custom: Vec<CustomProviderSummary>,
}

/// Outcome of a user-triggered `GET {base}/models` probe. The request carries
/// only the stored key; no book text is ever sent.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomTestResult {
    pub ok: bool,
    pub model_count: Option<usize>,
    /// Short human-readable outcome; never echoes the key or URL.
    pub message: String,
}
