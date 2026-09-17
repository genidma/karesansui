use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

const DEFAULT_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const MAX_RETRY_ATTEMPTS: u32 = 4;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Debug, Deserialize)]
struct ChatChunk {
    #[serde(default)]
    choices: Vec<ChunkChoice>,
}

#[derive(Debug, Deserialize)]
struct ChunkChoice {
    #[serde(default)]
    delta: Delta,
}

#[derive(Debug, Deserialize, Default)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
}

/// A parsed SSE line from an OpenAI-compatible streaming response.
enum SseEvent {
    Ignore,
    Done,
    Text(String),
}

fn parse_sse_line(line: &str) -> SseEvent {
    let line = line.trim();
    let Some(payload) = line.strip_prefix("data:") else {
        return SseEvent::Ignore;
    };
    let payload = payload.trim();
    if payload == "[DONE]" {
        return SseEvent::Done;
    }
    match serde_json::from_str::<ChatChunk>(payload) {
        Ok(chunk) => chunk
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.delta.content)
            .map(SseEvent::Text)
            .unwrap_or(SseEvent::Ignore),
        Err(_) => SseEvent::Ignore,
    }
}

pub struct LlmClient {
    client: reqwest::Client,
    api_key: String,
    pub model: String,
    pub api_url: String,
    max_tokens: u32,
}

impl LlmClient {
    pub fn new(api_key: String, model: String) -> Self {
        let api_url = std::env::var("LLM_API_URL")
            .or_else(|_| std::env::var("OPENROUTER_URL"))
            .unwrap_or_else(|_| DEFAULT_API_URL.to_string());
        let max_tokens = std::env::var("LLM_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(4000);
        log::info!("LLM API endpoint: {api_url}, max_tokens: {max_tokens}");
        Self {
            client: reqwest::Client::builder()
                .connect_timeout(CONNECT_TIMEOUT)
                .build()
                .unwrap_or_default(),
            api_key,
            model,
            api_url,
            max_tokens,
        }
    }

    fn is_openrouter(&self) -> bool {
        self.api_url.contains("openrouter.ai")
    }

    fn is_nvidia(&self) -> bool {
        self.api_url.contains("nvidia.com") || self.api_key.starts_with("nvapi-")
    }

    pub async fn call_raw(
        &self,
        system: &str,
        user: &str,
        temperature: f64,
        title: &str,
    ) -> Result<String> {
        let is_nvidia = self.is_nvidia();
        let mut body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user },
            ],
            "temperature": temperature,
            "stream": true,
            "max_tokens": self.max_tokens,
        });
        if is_nvidia {
            // NVIDIA reasoning models default to narrating their thinking aloud;
            // disable reasoning so they express the artwork directly as an artist.
            body["reasoning"] = json!({ "enabled": false });
        }

        let mut backoff = Duration::from_millis(1000);

        for attempt in 1..=MAX_RETRY_ATTEMPTS {
            let mut req = self
                .client
                .post(&self.api_url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&body);

            if self.is_openrouter() {
                req = req
                    .header("HTTP-Referer", "https://github.com/karesansui")
                    .header("X-Title", title);
            }

            if !self.is_nvidia() {
                log::info!("Sending LLM request to {} with model {} (attempt {attempt}/{MAX_RETRY_ATTEMPTS})...", self.api_url, self.model);
            }
            let resp = req.send().await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    if attempt < MAX_RETRY_ATTEMPTS {
                        log::warn!("Network error calling LLM API (attempt {attempt}/{MAX_RETRY_ATTEMPTS}): {e}. Retrying in {backoff:?}...");
                        tokio::time::sleep(backoff).await;
                        backoff *= 2;
                        continue;
                    }
                    return Err(anyhow::anyhow!("LLM API network error after {MAX_RETRY_ATTEMPTS} attempts: {e}"));
                }
            };

            let status = resp.status();
            if !status.is_success() {
                let retry_after = resp
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(Duration::from_secs);
                let err_body = resp.text().await.unwrap_or_default();
                let retryable = status == reqwest::StatusCode::TOO_MANY_REQUESTS
                    || status.is_server_error()
                    || status == reqwest::StatusCode::BAD_REQUEST;
                if retryable && attempt < MAX_RETRY_ATTEMPTS {
                    let wait = retry_after.unwrap_or(backoff);
                    log::warn!("LLM API returned status {status} (attempt {attempt}/{MAX_RETRY_ATTEMPTS}): {err_body}. Retrying in {wait:?}...");
                    tokio::time::sleep(wait).await;
                    backoff = (backoff * 2).min(Duration::from_secs(120));
                    continue;
                }
                return Err(anyhow::anyhow!("LLM API error (status {status}): {err_body}"));
            }

            // Stream the response so a slow/queued endpoint keeps the connection
            // warm and never trips a whole-response timeout.
            match self.read_stream(resp).await {
                Ok(content) if !content.trim().is_empty() => {
                    return Ok(strip_markdown_fence(&content));
                }
                Ok(_) => {
                    if attempt < MAX_RETRY_ATTEMPTS {
                        log::warn!("LLM returned an empty stream (attempt {attempt}/{MAX_RETRY_ATTEMPTS}). Retrying in {backoff:?}...");
                        tokio::time::sleep(backoff).await;
                        backoff *= 2;
                        continue;
                    }
                    return Err(anyhow::anyhow!("LLM returned an empty stream after {MAX_RETRY_ATTEMPTS} attempts"));
                }
                Err(e) => {
                    if attempt < MAX_RETRY_ATTEMPTS {
                        log::warn!("Stream error (attempt {attempt}/{MAX_RETRY_ATTEMPTS}): {e}. Retrying in {backoff:?}...");
                        tokio::time::sleep(backoff).await;
                        backoff *= 2;
                        continue;
                    }
                    return Err(anyhow::anyhow!("LLM stream failed after {MAX_RETRY_ATTEMPTS} attempts: {e}"));
                }
            }
        }

        Err(anyhow::anyhow!("Exceeded maximum retry attempts"))
    }

    /// Consume an OpenAI-compatible SSE stream, accumulating assistant text.
    /// A stalled stream (no bytes for `STREAM_IDLE_TIMEOUT`) is treated as an
    /// error so the caller can retry instead of hanging indefinitely.
    async fn read_stream(&self, resp: reqwest::Response) -> Result<String> {
        let mut stream = resp.bytes_stream();
        let mut buf = String::new();
        let mut content = String::new();
        let mut bytes = 0usize;
        loop {
            let next = tokio::time::timeout(STREAM_IDLE_TIMEOUT, stream.next())
                .await
                .map_err(|_| anyhow::anyhow!("stream idle for {STREAM_IDLE_TIMEOUT:?}"))?;
            let chunk = match next {
                Some(c) => c.context("stream read error")?,
                None => break,
            };
            bytes += chunk.len();
            buf.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(pos) = buf.find('\n') {
                let line: String = buf.drain(..=pos).collect();
                match parse_sse_line(&line) {
                    SseEvent::Text(t) => content.push_str(&t),
                    SseEvent::Done => {
                        log::info!("LLM creative response received ({bytes} bytes, streamed)");
                        return Ok(content);
                    }
                    SseEvent::Ignore => {}
                }
            }
        }
        log::info!("LLM creative response received ({bytes} bytes, streamed)");
        Ok(content)
    }
}

fn strip_markdown_fence(content: &str) -> String {
    content
        .trim()
        .strip_prefix("```json")
        .or_else(|| content.trim().strip_prefix("```"))
        .map(|s| {
            s.strip_suffix("```")
                .unwrap_or(s.trim())
                .trim()
                .to_string()
        })
        .unwrap_or_else(|| content.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_markdown_fence() {
        assert_eq!(strip_markdown_fence("```json\n{\"a\":1}\n```"), "{\"a\":1}");
        assert_eq!(strip_markdown_fence("```\n{\"a\":1}\n```"), "{\"a\":1}");
        assert_eq!(strip_markdown_fence("{\"a\":1}"), "{\"a\":1}");
        assert_eq!(strip_markdown_fence("```json\n{\"a\":1}"), "{\"a\":1}");
    }

    #[test]
    fn test_parse_sse_line() {
        let data = r#"data: {"choices":[{"delta":{"content":"hi"}}]}"#;
        assert!(matches!(parse_sse_line(data), SseEvent::Text(t) if t == "hi"));
        assert!(matches!(parse_sse_line("data: [DONE]"), SseEvent::Done));
        assert!(matches!(parse_sse_line(": OPENROUTER PROCESSING"), SseEvent::Ignore));
        assert!(matches!(parse_sse_line("data: not json"), SseEvent::Ignore));
        assert!(matches!(
            parse_sse_line(r#"data: {"choices":[{"delta":{}}]}"#),
            SseEvent::Ignore
        ));
    }
}
