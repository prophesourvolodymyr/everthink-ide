// providers/stub.rs — fallback provider when no API key is configured

use super::{LLMProvider, ProviderMessage, StreamEvent};
use async_trait::async_trait;
use tokio::sync::mpsc;

pub struct StubProvider;

#[async_trait]
impl LLMProvider for StubProvider {
    fn id(&self) -> &str {
        "stub"
    }

    fn model(&self) -> &str {
        "no-provider"
    }

    async fn complete(
        &self,
        _messages: &[ProviderMessage],
        tx: mpsc::UnboundedSender<StreamEvent>,
    ) -> anyhow::Result<()> {
        let config_path = crate::config::Config::config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~/.config/everthink/config.toml".into());

        let msg = format!(
            "No LLM provider is configured.\n\
             \n\
             Create {config_path} with your API key:\n\
             \n\
             [model]\n\
             provider = \"anthropic\"\n\
             model = \"claude-3-5-sonnet-20241022\"\n\
             \n\
             [providers.anthropic]\n\
             api_key = \"sk-ant-...\"\n\
             \n\
             Then restart Everthink IDE."
        );

        // Send the whole message as a single token, then Done
        let _ = tx.send(StreamEvent::Token(msg));
        let _ = tx.send(StreamEvent::Done);
        Ok(())
    }
}
