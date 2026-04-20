// providers/openai.rs — OpenAI Chat Completions via streaming API

use super::{LLMProvider, ProviderMessage, StreamEvent};
use async_trait::async_trait;
use futures::StreamExt;
use tokio::sync::mpsc;

pub struct OpenAIProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        OpenAIProvider {
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn id(&self) -> &str {
        "openai"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn complete(
        &self,
        messages: &[ProviderMessage],
        tx: mpsc::UnboundedSender<StreamEvent>,
    ) -> anyhow::Result<()> {
        // OpenAI accepts system messages inline
        let body_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": self.model,
            "stream": true,
            "messages": body_messages,
        });

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let _ = tx.send(StreamEvent::Error(format!("HTTP {status}: {body}")));
            return Ok(());
        }

        // Parse SSE stream line by line
        let mut byte_stream = response.bytes_stream();
        let mut buf = String::new();

        'outer: while let Some(chunk) = byte_stream.next().await {
            let bytes = chunk?;
            buf.push_str(&String::from_utf8_lossy(&bytes));

            loop {
                match buf.find('\n') {
                    None => break,
                    Some(pos) => {
                        let raw = buf[..pos].trim_end_matches('\r').to_string();
                        buf = buf[pos + 1..].to_string();

                        if let Some(data) = raw.strip_prefix("data: ") {
                            let data = data.trim();
                            if data == "[DONE]" {
                                break 'outer;
                            }
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                // Check for API-level error
                                if let Some(err) = val["error"]["message"].as_str() {
                                    let _ = tx.send(StreamEvent::Error(err.to_string()));
                                    return Ok(());
                                }
                                // Extract content delta
                                if let Some(content) =
                                    val["choices"][0]["delta"]["content"].as_str()
                                {
                                    if tx.send(StreamEvent::Token(content.to_string())).is_err() {
                                        return Ok(()); // receiver dropped
                                    }
                                }
                                // finish_reason = "stop" signals the end
                                if val["choices"][0]["finish_reason"].as_str() == Some("stop") {
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
            }
        }

        let _ = tx.send(StreamEvent::Done);
        Ok(())
    }
}
