# Providers — TODO (Phase 3)

## Context

Wire real LLM providers into the TUI. After this phase, the user can type a message and get
a real streamed response from Anthropic or OpenAI.

Config lives at `~/.config/everthink/config.toml`:
```toml
[model]
provider = "anthropic"
model = "claude-3-5-sonnet-20241022"

[providers.anthropic]
api_key = "sk-ant-..."

[providers.openai]
api_key = "sk-..."
```

If no config exists, a StubProvider replies with setup instructions.

---

## Todos

### src/config/mod.rs
- [x] `Config` struct (Deserialize/Serialize/Default)
- [x] `ModelConfig` — provider + model strings
- [x] `ProvidersConfig` — optional credentials per provider
- [x] `ProviderCreds` — api_key
- [x] `Config::load()` — reads file or returns Default
- [x] `Config::save()` — writes to disk
- [x] `Config::config_path()` — ~/.config/everthink/config.toml

### src/providers/mod.rs
- [x] `ProviderMessage { role: String, content: String }`
- [x] `StreamEvent { Token(String), Done, Error(String) }`
- [x] `LLMProvider` async trait
- [x] `ProviderRegistry::new(&config)`
- [x] `ProviderRegistry::default_provider() -> Arc<dyn LLMProvider>`

### src/providers/stub.rs
- [x] Replies with "No provider configured" + setup instructions
- [x] Used as fallback when no API key in config

### src/providers/anthropic.rs
- [x] POST to https://api.anthropic.com/v1/messages
- [x] SSE streaming — parse `content_block_delta` events
- [x] System messages extracted as top-level `system` field
- [x] Error handling: HTTP non-200, API error events

### src/providers/openai.rs
- [x] POST to https://api.openai.com/v1/chat/completions
- [x] SSE streaming — parse `choices[0].delta.content`
- [x] `[DONE]` sentinel detection
- [x] Error handling: HTTP non-200

### src/tui/mod.rs (async updates)
- [x] `run()` → `async fn run()`
- [x] Load config + create ProviderRegistry at startup
- [x] `App::provider: Arc<dyn LLMProvider>`
- [x] `App::stream_rx: Option<mpsc::UnboundedReceiver<StreamEvent>>`
- [x] `App::is_streaming: bool`
- [x] `App::streaming_msg_idx: Option<usize>`
- [x] `App::system_prompt: String`
- [x] `App::send_message()` — push user msg, spawn provider task, create stream channel
- [x] `App::build_provider_messages()` — system prompt + non-system chat history
- [x] `handle_stream_event()` — append tokens, handle Done/Error
- [x] Event loop: `tokio::select!` with EventStream + stream channel + 500ms tick

### src/tui/chat.rs
- [x] Streaming cursor `▊` appended to last assistant line when `app.is_streaming`

### Cargo.toml
- [x] `crossterm = { version = "0.28", features = ["event-stream"] }`

---

## Dependencies

- Phase 2 (complete)

---

## Blockers

- None (API keys are user-provided at runtime via config)

---

## Definition of Done

- `everthink` with no config shows stub setup message
- `everthink` with valid anthropic key → messages get real streamed responses
- `everthink` with valid openai key → messages get real streamed responses
- Streaming cursor `▊` visible while response is incoming
- `cargo build` zero errors
