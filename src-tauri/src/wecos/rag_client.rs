//! WEcoS RAG Client: Engine 1 (Intel NPU :8087) Dense Embeddings & Command Center Retrieval.
//!
//! Enforces negative invariant: Never run embeddings on Engine 3 (eGPU R9700).
//! Embeddings are pinned strictly to Engine 1 (Intel AI Boost NPU).
//! If offline, gracefully returns None/Offline error so translation can continue unblocked.

use std::time::Duration;
use serde::{Deserialize, Serialize};
use super::{NPU_EMBED_URL, COMMAND_CENTER_URL};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub embedding: Vec<f32>,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub data: Vec<EmbeddingData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagChunk {
    pub id: String,
    pub text: String,
    pub score: f32,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagQueryRequest {
    pub query: String,
    pub top_k: usize,
    pub collections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagQueryResponse {
    pub chunks: Vec<RagChunk>,
}

pub struct WecosRagClient {
    client: reqwest::blocking::Client,
    npu_url: String,
    retrieval_url: String,
}

impl Default for WecosRagClient {
    fn default() -> Self {
        Self::new(Duration::from_millis(1000))
    }
}

impl WecosRagClient {
    pub fn new(timeout: Duration) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .connect_timeout(Duration::from_millis(150))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        Self {
            client,
            npu_url: NPU_EMBED_URL.to_string(),
            retrieval_url: COMMAND_CENTER_URL.to_string(),
        }
    }

    /// Generates 1024-d BGE-M3 embedding using Engine 1 (Intel AI Boost NPU).
    pub fn get_embedding(&self, text: &str) -> Result<Vec<f32>, String> {
        let url = format!("{}/v1/embeddings", self.npu_url);
        let req = EmbeddingRequest {
            model: "BAAI/bge-m3".to_string(),
            input: text.to_string(),
        };

        let resp = self.client.post(&url)
            .json(&req)
            .send()
            .map_err(|e| format!("NPU embedder request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("NPU embedder returned HTTP {}", resp.status()));
        }

        let body: EmbeddingResponse = resp.json()
            .map_err(|e| format!("Failed to parse NPU embedding response: {}", e))?;

        body.data.into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| "Empty embedding data from NPU".to_string())
    }

    /// Fetches Top-K reference glossary / historical context chunks from WEcoS Command Center.
    pub fn query_context(&self, query_text: &str, top_k: usize) -> Result<Vec<RagChunk>, String> {
        let url = format!("{}/api/retrieval/query", self.retrieval_url);
        let req = RagQueryRequest {
            query: query_text.to_string(),
            top_k,
            collections: vec!["wecos_all".to_string()],
        };

        let resp = self.client.post(&url)
            .json(&req)
            .send()
            .map_err(|e| format!("WEcoS retrieval request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("WEcoS retrieval returned HTTP {}", resp.status()));
        }

        let body: RagQueryResponse = resp.json()
            .map_err(|e| format!("Failed to parse retrieval response: {}", e))?;

        Ok(body.chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rag_client_creation_and_offline_fail_graceful() {
        let client = WecosRagClient::new(Duration::from_millis(50));
        // Calling non-running local endpoint should return graceful Err, never panic
        let emb_res = client.get_embedding("Test passage");
        assert!(emb_res.is_err());
        let rag_res = client.query_context("Test query", 2);
        assert!(rag_res.is_err());
    }
}
