//! WEcoS Tri-Engine Integration Bridge & Circuit Breaker.
//!
//! Provides capability probing, RAG context enrichment, and Dawei Guard
//! pre/post-LLM safety validation for Sagar, bound to WEcoS Silicon Topology:
//! - Engine 1: Intel AI Boost NPU (:8087) -> BGE-M3 1024-d Dense Embeddings
//! - Engine 2: Intel Arc 140T OpenVINO (:8089) -> Dawei Guard v3 pre-LLM & Burmese orthography
//! - Engine 3: AMD Radeon AI PRO R9700 eGPU (:11434 / :18982) -> Text generation / vLLM MoE
//! - Command Center (:7860) -> Persona retrieval & LanceDB master index
//!
//! Enforces negative invariant: 150ms strict circuit-breaker timeout on all probes;
//! if daemons are offline, degrades gracefully to standalone mode without blocking UI.

pub mod dawei_client;
pub mod rag_client;

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

pub const DEFAULT_CIRCUIT_BREAKER_TIMEOUT_MS: u64 = 150;
pub const NPU_EMBED_URL: &str = "http://127.0.0.1:8087";
pub const DAWEI_GUARD_URL: &str = "http://127.0.0.1:8089";
pub const VLLM_TIER0_URL: &str = "http://127.0.0.1:18982";
pub const COMMAND_CENTER_URL: &str = "http://127.0.0.1:7860";
pub const OLLAMA_ENGINE3_URL: &str = "http://127.0.0.1:11434";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WecosCapabilities {
    pub npu_available: bool,
    pub dawei_guard_available: bool,
    pub vllm_available: bool,
    pub command_center_available: bool,
    pub ollama_available: bool,
    pub is_connected: bool,
    pub probe_elapsed_ms: u128,
}

impl Default for WecosCapabilities {
    fn default() -> Self {
        Self {
            npu_available: false,
            dawei_guard_available: false,
            vllm_available: false,
            command_center_available: false,
            ollama_available: false,
            is_connected: false,
            probe_elapsed_ms: 0,
        }
    }
}

/// Probes all WEcoS daemons with a strict per-endpoint circuit-breaker timeout.
///
/// Guaranteed to return within `timeout_ms * num_probes` (or faster with concurrent/individual budgets).
/// Never panics, never blocks the caller indefinitely.
pub fn probe_capabilities(timeout_ms: u64) -> WecosCapabilities {
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    let client = match reqwest::blocking::Client::builder()
        .timeout(timeout)
        .connect_timeout(timeout)
        .build()
    {
        Ok(c) => c,
        Err(_) => return WecosCapabilities::default(),
    };

    let check_endpoint = |url: &str| -> bool {
        client.get(url).send().map(|res| res.status().is_success() || res.status().is_redirection()).unwrap_or(false)
    };

    let npu_available = check_endpoint(&format!("{}/health", NPU_EMBED_URL))
        || check_endpoint(&format!("{}/v1/models", NPU_EMBED_URL));

    let dawei_guard_available = check_endpoint(&format!("{}/health", DAWEI_GUARD_URL))
        || check_endpoint(&format!("{}/readiness", DAWEI_GUARD_URL));

    let vllm_available = check_endpoint(&format!("{}/v1/models", VLLM_TIER0_URL));

    let command_center_available = check_endpoint(&format!("{}/readiness", COMMAND_CENTER_URL));

    let ollama_available = check_endpoint(&format!("{}/api/tags", OLLAMA_ENGINE3_URL));

    let is_connected = npu_available || dawei_guard_available || vllm_available || command_center_available || ollama_available;
    let elapsed = start.elapsed().as_millis();

    WecosCapabilities {
        npu_available,
        dawei_guard_available,
        vllm_available,
        command_center_available,
        ollama_available,
        is_connected,
        probe_elapsed_ms: elapsed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_capabilities_offline_fallback() {
        // Probe with a tight 50ms budget against non-existent or localhost ports
        let caps = probe_capabilities(50);
        // Should not panic, should return valid struct within reasonable duration
        assert!(caps.probe_elapsed_ms <= 5000);
        // When daemons are not running or mocked, is_connected accurately reflects individual flags
        assert_eq!(caps.is_connected, caps.npu_available || caps.dawei_guard_available || caps.vllm_available || caps.command_center_available || caps.ollama_available);
    }
}
