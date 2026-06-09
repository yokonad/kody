use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Re-export scanner types
pub use crate::scanner::{Severity, Vulnerability};

pub mod openai;
pub mod offline;

pub use openai::OpenAiProvider;
pub use offline::OfflineProvider;

// Types for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub target: String,
    pub open_ports: Vec<u16>,
    pub services: Vec<ServiceInfo>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub raw_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub port: u16,
    pub service: String,
    pub version: Option<String>,
}

/// AI provider trait - implementations must be async and thread-safe
#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn analyze(&self, scan_result: ScanResult) -> Result<String, AiError>;
    #[allow(dead_code)]
    fn name(&self) -> &str;
    #[allow(dead_code)]
    fn is_configured(&self) -> bool;
}

/// AI-related errors
#[derive(Debug)]
pub enum AiError {
    NotConfigured,
    ApiError(String),
    NetworkError(String),
    ParseError(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::NotConfigured => write!(f, "AI provider not configured"),
            AiError::ApiError(s) => write!(f, "API error: {}", s),
            AiError::NetworkError(s) => write!(f, "Network error: {}", s),
            AiError::ParseError(s) => write!(f, "Parse error: {}", s),
        }
    }
}

impl std::error::Error for AiError {}

/// Factory: pick the AI provider from the key + optional provider hint.
///
/// If `provider_name` is not given, the provider is auto-detected from the key
/// format (Anthropic / OpenAI / Google). With no usable key, falls back to the
/// fully-offline analyzer.
pub fn create_provider(api_key: Option<String>, provider_name: Option<String>) -> Box<dyn AiProvider> {
    if let Some(key) = api_key {
        if !key.is_empty() {
            let provider = provider_name
                .or_else(|| crate::config::detect_provider(&key).map(|s| s.to_string()))
                .unwrap_or_else(|| "openai".to_string());
            return Box::new(OpenAiProvider::new(Some(key), Some(provider)));
        }
    }
    Box::new(OfflineProvider::new())
}