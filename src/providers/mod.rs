// providers/mod.rs — LLMProvider trait, StreamEvent, ProviderRegistry

pub mod anthropic;
pub mod openai;
pub mod openrouter;
pub mod stub;

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

// ─── Message and event types ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ProviderMessage {
    pub role: String,    // "system" | "user" | "assistant"
    pub content: String,
}

#[derive(Debug)]
pub enum StreamEvent {
    /// A text chunk arriving from the model
    Token(String),
    /// The model has finished generating
    Done,
    /// An error occurred during streaming
    Error(String),
}

// ─── Provider trait ───────────────────────────────────────────────────────────

#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Short identifier, e.g. "anthropic", "openai"
    fn id(&self) -> &str;
    /// Model name shown in status bar
    fn model(&self) -> &str;
    /// Stream a completion. Sends StreamEvent::Token* then Done (or Error).
    async fn complete(
        &self,
        messages: &[ProviderMessage],
        tx: mpsc::UnboundedSender<StreamEvent>,
    ) -> anyhow::Result<()>;
}

// ─── Registry ────────────────────────────────────────────────────────────────

pub struct ProviderRegistry {
    providers: Vec<Arc<dyn LLMProvider>>,
    default_id: String,
}

impl ProviderRegistry {
    pub fn new(config: &crate::config::Config) -> Self {
        let mut providers: Vec<Arc<dyn LLMProvider>> = Vec::new();

        // Anthropic
        if let Some(creds) = &config.providers.anthropic {
            let key = creds.api_key.trim().to_string();
            if !key.is_empty() {
                providers.push(Arc::new(anthropic::AnthropicProvider::new(
                    key,
                    config.model.model.clone(),
                )));
            }
        }

        // OpenAI
        if let Some(creds) = &config.providers.openai {
            let key = creds.api_key.trim().to_string();
            if !key.is_empty() {
                providers.push(Arc::new(openai::OpenAIProvider::new(
                    key,
                    if config.model.provider == "openai" {
                        config.model.model.clone()
                    } else {
                        "gpt-4o".into()
                    },
                )));
            }
        }

        // OpenRouter
        if let Some(creds) = &config.providers.openrouter {
            let key = creds.api_key.trim().to_string();
            if !key.is_empty() {
                providers.push(Arc::new(openrouter::OpenRouterProvider::new(
                    key,
                    if config.model.provider == "openrouter" {
                        config.model.model.clone()
                    } else {
                        "openai/gpt-4o".into()
                    },
                )));
            }
        }

        // Fallback stub if nothing is configured
        if providers.is_empty() {
            providers.push(Arc::new(stub::StubProvider));
        }

        let default_id = config.model.provider.clone();
        ProviderRegistry { providers, default_id }
    }

    /// Returns the configured default provider, or the first available one.
    pub fn default_provider(&self) -> Arc<dyn LLMProvider> {
        self.providers
            .iter()
            .find(|p| p.id() == self.default_id)
            .or_else(|| self.providers.first())
            .cloned()
            .expect("at least one provider always exists (stub fallback)")
    }

    /// Look up a provider by id.
    pub fn get(&self, id: &str) -> Option<Arc<dyn LLMProvider>> {
        self.providers.iter().find(|p| p.id() == id).cloned()
    }

    /// All registered providers.
    pub fn list(&self) -> Vec<Arc<dyn LLMProvider>> {
        self.providers.clone()
    }
}
