use async_trait::async_trait;
use crate::ai::{AiProvider, AiError, ScanResult};

pub struct OpenAiProvider {
    api_key: Option<String>,
    provider: String,
}

impl OpenAiProvider {
    pub fn new(api_key: Option<String>, provider: Option<String>) -> Self {
        Self {
            api_key,
            provider: provider.unwrap_or_else(|| "openai".to_string()),
        }
    }

    fn build_prompt(&self, scan_result: &ScanResult) -> String {
        let mut prompt = String::from("You are a cybersecurity expert analyzing vulnerability scan results.\n\n");
        prompt.push_str(&format!("Target: {}\n", scan_result.target));
        prompt.push_str(&format!("Open ports: {:?}\n", scan_result.open_ports));
        prompt.push_str("Services:\n");
        for svc in &scan_result.services {
            prompt.push_str(&format!("  - Port {}: {} {:?}\n", svc.port, svc.service, svc.version));
        }
        prompt.push_str("Vulnerabilities found:\n");
        for vuln in &scan_result.vulnerabilities {
            prompt.push_str(&format!("  - [{}] {} (port {})\n", vuln.severity, vuln.description, vuln.affected_port));
        }
        prompt.push_str("\nProvide a detailed analysis and recommended actions.");
        prompt
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn analyze(&self, scan_result: ScanResult) -> Result<String, AiError> {
        let api_key = self.api_key.as_ref().ok_or(AiError::NotConfigured)?;

        let prompt = self.build_prompt(&scan_result);
        let provider = self.provider.as_str();

        let client = reqwest::Client::new();

        // Each provider gets its own endpoint, request shape, and auth header.
        let (url, body, request) = match provider {
            "anthropic" => {
                let url = "https://api.anthropic.com/v1/messages".to_string();
                let body = serde_json::json!({
                    "model": "claude-opus-4-8",
                    "max_tokens": 2048,
                    "messages": [{"role": "user", "content": prompt}]
                });
                let req = client.post(&url)
                    .header("Content-Type", "application/json")
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01");
                (url, body, req)
            }
            "google" => {
                let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent".to_string();
                let body = serde_json::json!({
                    "contents": [{"parts": [{"text": prompt}]}]
                });
                let req = client.post(&url)
                    .header("Content-Type", "application/json")
                    .header("x-goog-api-key", api_key);
                (url, body, req)
            }
            _ => {
                // OpenAI-compatible chat completions.
                let url = "https://api.openai.com/v1/chat/completions".to_string();
                let body = serde_json::json!({
                    "model": "gpt-4o",
                    "messages": [{"role": "user", "content": prompt}]
                });
                let req = client.post(&url)
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", api_key));
                (url, body, req)
            }
        };
        let _ = url;

        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AiError::ApiError(format!("API returned status: {}", response.status())));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AiError::ParseError(e.to_string()))?;

        // Parse the response shape per provider.
        let json = serde_json::from_str::<serde_json::Value>(&response_text).ok();
        if let Some(json) = json {
            let extracted = match provider {
                "anthropic" => json.get("content")
                    .and_then(|c| c.as_array()).and_then(|a| a.first())
                    .and_then(|o| o.get("text")).and_then(|t| t.as_str()),
                "google" => json.get("candidates")
                    .and_then(|c| c.as_array()).and_then(|a| a.first())
                    .and_then(|o| o.get("content")).and_then(|c| c.get("parts"))
                    .and_then(|p| p.as_array()).and_then(|a| a.first())
                    .and_then(|o| o.get("text")).and_then(|t| t.as_str()),
                _ => json.get("choices")
                    .and_then(|c| c.as_array()).and_then(|a| a.first())
                    .and_then(|o| o.get("message")).and_then(|m| m.get("content"))
                    .and_then(|t| t.as_str()),
            };
            if let Some(text) = extracted {
                return Ok(text.to_string());
            }
        }

        Ok(response_text)
    }

    fn name(&self) -> &str {
        &self.provider
    }

    fn is_configured(&self) -> bool {
        self.api_key.is_some() && !self.api_key.as_ref().unwrap().is_empty()
    }
}