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
        let model = if self.provider == "anthropic" { "claude-3" } else { "gpt-4" };

        let client = reqwest::Client::new();
        let url = if self.provider == "anthropic" {
            "https://api.anthropic.com/v1/messages"
        } else {
            "https://api.openai.com/v1/chat/completions"
        };

        let request_body = if self.provider == "anthropic" {
            serde_json::json!({
                "model": model,
                "max_tokens": 2048,
                "messages": [{"role": "user", "content": prompt}]
            })
        } else {
            serde_json::json!({
                "model": model,
                "messages": [{"role": "user", "content": prompt}]
            })
        };

        let mut request = client.post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json");

        if self.provider == "anthropic" {
            request = request.header("x-api-key", api_key);
        }

        let response = request
            .json(&request_body)
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

        // Parse response based on provider
        if self.provider == "anthropic" {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                if let Some(content) = json.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first())
                    .and_then(|obj| obj.get("text")).and_then(|t| t.as_str()) {
                    return Ok(content.to_string());
                }
            }
        } else {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                if let Some(content) = json.get("choices").and_then(|c| c.as_array()).and_then(|arr| arr.first())
                    .and_then(|obj| obj.get("message")).and_then(|m| m.get("content")).and_then(|t| t.as_str()) {
                    return Ok(content.to_string());
                }
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