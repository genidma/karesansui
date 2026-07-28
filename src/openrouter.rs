use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

const DEFAULT_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const MAX_RETRY_ATTEMPTS: u32 = 4;

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: String,
}

pub struct LlmClient {
    client: reqwest::Client,
    api_key: String,
    pub model: String,
    pub api_url: String,
}

impl LlmClient {
    pub fn new(api_key: String, model: String) -> Self {
        let api_url = std::env::var("LLM_API_URL")
            .or_else(|_| std::env::var("OPENROUTER_URL"))
            .unwrap_or_else(|_| DEFAULT_API_URL.to_string());
        log::info!("LLM API endpoint: {api_url}");
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            api_url,
        }
    }

    fn is_openrouter(&self) -> bool {
        self.api_url.contains("openrouter.ai")
    }

    pub async fn call_raw(
        &self,
        system: &str,
        user: &str,
        temperature: f64,
        title: &str,
    ) -> Result<String> {
        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user },
            ],
            "temperature": temperature,
        });

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

            log::info!("Sending LLM request to {} with model {} (attempt {attempt}/{MAX_RETRY_ATTEMPTS})...", self.api_url, self.model);
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

            let chat_resp: ChatResponse = resp.json().await.context("Failed to parse LLM JSON response")?;

            let content = chat_resp
                .choices
                .into_iter()
                .next()
                .map(|c| c.message.content)
                .ok_or_else(|| anyhow::anyhow!("No choices returned from LLM API"))?;

            return Ok(strip_markdown_fence(&content));
        }

        Err(anyhow::anyhow!("Exceeded maximum retry attempts"))
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
}
