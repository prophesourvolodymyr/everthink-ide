// providers/anthropic.rs — Anthropic Claude via streaming Messages API

use super::{LLMProvider, ProviderMessage, StreamEvent};
use async_trait::async_trait;
use futures::StreamExt;
use tokio::sync::mpsc;

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        AnthropicProvider {
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    fn id(&self) -> &str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn complete(
        &self,
        messages: &[ProviderMessage],
        tx: mpsc::UnboundedSender<StreamEvent>,
    ) -> anyhow::Result<()> {
        // Anthropic requires system messages as a top-level field
        let system: String = messages
            .iter()
            .filter(|m| m.role == "system")
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let chat_messages: Vec<serde_json::Value> = messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        if chat_messages.is_empty() {
            let _ = tx.send(StreamEvent::Error("No user/assistant messages to send.".into()));
            return Ok(());
        }

        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": 8096,
            "stream": true,
            "messages": chat_messages,
        });
        if !system.is_empty() {
            body["system"] = serde_json::Value::String(system);
        }

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
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

            // Process all complete lines from the buffer
            loop {
                match buf.find('\n') {
                    None => break, // wait for more bytes
                    Some(pos) => {
                        let raw = buf[..pos].trim_end_matches('\r').to_string();
                        buf = buf[pos + 1..].to_string();

                        if let Some(data) = raw.strip_prefix("data: ") {
                            let data = data.trim();
                            if data == "[DONE]" {
                                break 'outer;
                            }
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                match val["type"].as_str().unwrap_or("") {
                                    "content_block_delta" => {
                                        if let Some(text) = val["delta"]["text"].as_str() {
                                            if tx.send(StreamEvent::Token(text.to_string())).is_err() {
                                                return Ok(()); // receiver was dropped
                                            }
                                        }
                                    }
                                    "message_stop" => {
                                        let _ = tx.send(StreamEvent::Done);
                                        return Ok(());
                                    }
                                    "error" => {
                                        let msg = val["error"]["message"]
                                            .as_str()
                                            .unwrap_or("Unknown API error");
                                        let _ = tx.send(StreamEvent::Error(msg.to_string()));
                                        return Ok(());
                                    }
                                    _ => {}
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
