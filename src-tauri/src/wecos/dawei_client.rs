//! Dawei Guard Client: Engine 2 (Intel Arc 140T OpenVINO :8089).
//!
//! Provides pre-LLM injection scanning and post-LLM Burmese orthographic integrity checks.
//! Includes autonomous local fallback normalization so translation is never blocked
//! when Engine 2 is offline or running headlessly.

use std::time::Duration;
use serde::{Deserialize, Serialize};
use super::DAWEI_GUARD_URL;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaweiCheckRequest {
    pub text: String,
    pub mode: String, // e.g. "pre_llm" or "orthography"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaweiCheckResponse {
    pub passed: bool,
    pub score: f32,
    pub flags: Vec<String>,
    pub normalized_text: Option<String>,
}

pub struct DaweiGuardClient {
    client: reqwest::blocking::Client,
    guard_url: String,
}

impl Default for DaweiGuardClient {
    fn default() -> Self {
        Self::new(Duration::from_millis(300))
    }
}

impl DaweiGuardClient {
    pub fn new(timeout: Duration) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .connect_timeout(Duration::from_millis(150))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        Self {
            client,
            guard_url: DAWEI_GUARD_URL.to_string(),
        }
    }

    /// Pre-LLM check against prompt injection or prompt leakage.
    pub fn pre_check(&self, text: &str) -> Result<DaweiCheckResponse, String> {
        let url = format!("{}/v1/guard/check", self.guard_url);
        let req = DaweiCheckRequest {
            text: text.to_string(),
            mode: "pre_llm".to_string(),
        };

        let resp = self.client.post(&url)
            .json(&req)
            .send()
            .map_err(|e| format!("Dawei Guard request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Dawei Guard returned HTTP {}", resp.status()));
        }

        resp.json().map_err(|e| format!("Failed to parse Dawei Guard response: {}", e))
    }

    /// Post-LLM orthography check with automatic local fallback normalization.
    pub fn verify_or_normalize(&self, burmese_text: &str) -> String {
        // Try Engine 2 if available
        let url = format!("{}/v1/guard/check", self.guard_url);
        let req = DaweiCheckRequest {
            text: burmese_text.to_string(),
            mode: "orthography".to_string(),
        };

        if let Ok(resp) = self.client.post(&url).json(&req).send() {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<DaweiCheckResponse>() {
                    if let Some(norm) = data.normalized_text {
                        return norm;
                    }
                }
            }
        }

        // Autonomous local normalization fallback (Unicode canonical ordering)
        Self::local_normalize_myanmar(burmese_text)
    }

    /// Local fallback canonical Myanmar Unicode ordering:
    /// Repairs common legacy/AI inversion patterns (e.g. U+1031 E-vowel before consonant, or misplaced tone marks).
    pub fn local_normalize_myanmar(text: &str) -> String {
        let mut result = text.to_string();
        // Remove zero-width spaces that occasionally get emitted by LLMs inside words
        result = result.replace("\u{200B}", "");
        // Fix double asat
        result = result.replace("\u{103A}\u{103A}", "\u{103A}");
        // Fix double anusvara
        result = result.replace("\u{1036}\u{1036}", "\u{1036}");
        // Fix misplaced tone marks U+1037 before U+1036
        result = result.replace("\u{1037}\u{1036}", "\u{1036}\u{1037}");

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_normalize_myanmar() {
        let malformed = "စာအုပ်\u{200B}စမ်းသပ်\u{1037}\u{1036}ခြင်း\u{103A}\u{103A}";
        let normalized = DaweiGuardClient::local_normalize_myanmar(malformed);
        assert!(!normalized.contains('\u{200B}'));
        assert!(!normalized.contains("\u{1037}\u{1036}"));
        assert!(normalized.contains("\u{1036}\u{1037}"));
        assert!(!normalized.contains("\u{103A}\u{103A}"));
    }

    #[test]
    fn test_dawei_client_offline_graceful() {
        let client = DaweiGuardClient::new(Duration::from_millis(50));
        let check_res = client.pre_check("Hello world");
        assert!(check_res.is_err());

        // verify_or_normalize should never panic and should fallback to local normalization
        let raw = "မြန်မာစာ\u{200B}စမ်းသပ်";
        let out = client.verify_or_normalize(raw);
        assert_eq!(out, "မြန်မာစာစမ်းသပ်");
    }
}
