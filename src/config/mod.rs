// config/mod.rs — global configuration loaded from ~/.config/everthink/config.toml

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─── Top-level config ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub model: ModelConfig,
    #[serde(default)]
    pub providers: ProvidersConfig,
}

// ─── Model section ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelConfig {
    /// Which provider to use by default ("anthropic", "openai", …)
    pub provider: String,
    /// Model name within that provider
    pub model: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        ModelConfig {
            provider: "anthropic".into(),
            model: "claude-3-5-sonnet-20241022".into(),
        }
    }
}

// ─── Providers section ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ProvidersConfig {
    pub anthropic: Option<ProviderCreds>,
    pub openai: Option<ProviderCreds>,
    pub openrouter: Option<ProviderCreds>,
    pub ollama: Option<OllamaConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProviderCreds {
    pub api_key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OllamaConfig {
    #[serde(default = "default_ollama_url")]
    pub base_url: String,
    pub model: Option<String>,
}

fn default_ollama_url() -> String {
    "http://localhost:11434".into()
}

// ─── Load / Save ─────────────────────────────────────────────────────────────

impl Config {
    /// Returns the config file path, e.g. ~/.config/everthink/config.toml
    pub fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "everthink")
            .map(|dirs| dirs.config_dir().join("config.toml"))
    }

    /// Load config from disk. Returns Default if the file doesn't exist.
    pub fn load() -> Result<Self> {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Ok(Config::default()),
        };

        if !path.exists() {
            return Ok(Config::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Write config back to disk, creating parent dirs if needed.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config path"))?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Returns a human-readable summary for the TUI welcome message.
    /// Shows the config path and masked key prefix so the user can verify what loaded.
    pub fn diagnostic(&self) -> String {
        let path = Self::config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "unknown".into());

        let key_info = if let Some(or) = &self.providers.openrouter {
            let k = or.api_key.trim();
            if k.is_empty() {
                "openrouter key: EMPTY".into()
            } else {
                format!("openrouter key: {}…", &k[..k.len().min(12)])
            }
        } else if let Some(ai) = &self.providers.anthropic {
            let k = ai.api_key.trim();
            if k.is_empty() {
                "anthropic key: EMPTY".into()
            } else {
                format!("anthropic key: {}…", &k[..k.len().min(12)])
            }
        } else if let Some(oa) = &self.providers.openai {
            let k = oa.api_key.trim();
            if k.is_empty() {
                "openai key: EMPTY".into()
            } else {
                format!("openai key: {}…", &k[..k.len().min(12)])
            }
        } else {
            "no provider key found".into()
        };

        format!("Config: {path}\n{key_info}  model: {}/{}", self.model.provider, self.model.model)
    }

    /// Write a sample config file if none exists.
    pub fn write_sample_if_missing() -> Result<()> {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Ok(()),
        };
        if path.exists() {
            return Ok(());
        }

        let sample = r#"# Everthink IDE — configuration
# ~/.config/everthink/config.toml

[model]
provider = "anthropic"
model = "claude-3-5-sonnet-20241022"

# [providers.anthropic]
# api_key = "sk-ant-..."

# [providers.openai]
# api_key = "sk-..."
"#;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, sample)?;
        Ok(())
    }
}
