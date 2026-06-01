use async_trait::async_trait;

// Re-export scanner types for convenience
pub use crate::scanner::{Severity, Vulnerability, ScanResult, ServiceInfo};

pub mod openai;
pub mod offline;

pub use openai::OpenAiProvider;
pub use offline::OfflineProvider;

/// AI provider trait - implementations must be async and thread-safe
#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn analyze(&self, scan_result: ScanResult) -> Result<String, AiError>;
    fn name(&self) -> &str;
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

/// Factory function to create the appropriate AI provider based on configuration
pub fn create_provider(api_key: Option<String>, provider_name: Option<String>) -> Box<dyn AiProvider> {
    if let Some(key) = api_key {
        if !key.is_empty() {
            match provider_name.as_deref() {
                Some("anthropic") => return Box::new(OpenAiProvider::new(Some(key), Some("anthropic".to_string()))),
                _ => return Box::new(OpenAiProvider::new(Some(key), None)),
            }
        }
    }
    Box::new(OfflineProvider::new())
}