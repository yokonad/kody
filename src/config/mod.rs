use std::path::PathBuf;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const CONFIG_DIR: &str = ".kody";
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub ai_provider: Option<String>,
    pub ai_key: Option<String>,
    pub output_format: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ai_provider: None,
            ai_key: None,
            output_format: Some("text".to_string()),
        }
    }
}

/// Auto-detect the AI provider from the shape of an API key.
///
/// Kody is not tied to one vendor: you paste a key and the system figures out
/// who it belongs to. Returns `None` for unrecognized formats.
pub fn detect_provider(key: &str) -> Option<&'static str> {
    let k = key.trim();
    if k.starts_with("sk-ant-") {
        Some("anthropic")
    } else if k.starts_with("AIza") {
        Some("google")
    } else if k.starts_with("sk-") || k.starts_with("sess-") {
        Some("openai")
    } else {
        None
    }
}

impl Settings {
    fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Cannot find home directory")?;
        Ok(home.join(CONFIG_DIR).join(CONFIG_FILE))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Settings::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let settings: Settings = toml::from_str(&content)?;
        // Override with environment variables if set
        Ok(settings.with_env_overrides())
    }

    fn with_env_overrides(mut self) -> Self {
        if let Ok(key) = std::env::var("KODY_AI_KEY") {
            if !key.is_empty() {
                self.ai_key = Some(key);
            }
        }
        if let Ok(provider) = std::env::var("KODY_AI_PROVIDER") {
            if !provider.is_empty() {
                self.ai_provider = Some(provider);
            }
        }
        self
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[allow(dead_code)]
pub struct Config;

impl Config {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn ai_enabled(&self) -> bool {
        std::env::var("KODY_AI_KEY").is_ok()
            || Settings::load()
                .map(|s| s.ai_key.is_some())
                .unwrap_or(false)
    }
}

// For backward compatibility
#[allow(dead_code)]
pub type ConfigType = Config;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert_eq!(settings.output_format, Some("text".to_string()));
        assert!(settings.ai_key.is_none());
    }

    #[test]
    fn test_detect_provider() {
        assert_eq!(detect_provider("sk-ant-api03-abc"), Some("anthropic"));
        assert_eq!(detect_provider("AIzaSyabc123"), Some("google"));
        assert_eq!(detect_provider("sk-proj-abc"), Some("openai"));
        assert_eq!(detect_provider("garbage"), None);
    }
}