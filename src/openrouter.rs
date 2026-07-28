use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const MAX_RETRY_ATTEMPTS: u32 = 4;

#[derive(Debug, Deserialize)]
struct ORResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessageOut,
}

#[derive(Debug, Deserialize)]
struct ChatMessageOut {
    content: String,
}

pub struct LlmClient {
    client: reqwest::Client,
    api_key: String,
    pub model: String,
}

impl LlmClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
        }
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
            let resp = self
                .client
                .post(OPENROUTER_URL)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .header("HTTP-Referer", "https://github.com/karesansui")
                .header("X-Title", title)
                .json(&body)
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    if attempt < MAX_RETRY_ATTEMPTS {
                        log::warn!("Network error calling OpenRouter (attempt {attempt}/{MAX_RETRY_ATTEMPTS}): {e}. Retrying in {backoff:?}...");
                        tokio::time::sleep(backoff).await;
                        backoff *= 2;
                        continue;
                    }
                    return Err(anyhow::anyhow!("OpenRouter network error after {MAX_RETRY_ATTEMPTS} attempts: {e}"));
                }
            };

            let status = resp.status();
            if !status.is_success() {
                let err_body = resp.text().await.unwrap_or_default();
                let retryable = status == reqwest::StatusCode::TOO_MANY_REQUESTS
                    || status.is_server_error()
                    || status == reqwest::StatusCode::BAD_REQUEST;
                if retryable && attempt < MAX_RETRY_ATTEMPTS {
                    log::warn!("OpenRouter API returned status {status} (attempt {attempt}/{MAX_RETRY_ATTEMPTS}): {err_body}. Retrying in {backoff:?}...");
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                    continue;
                }
                return Err(anyhow::anyhow!("OpenRouter API error (status {status}): {err_body}"));
            }

            let or_resp: ORResponse = resp.json().await.context("Failed to parse OpenRouter JSON response")?;

            let content = or_resp
                .choices
                .into_iter()
                .next()
                .map(|c| c.message.content)
                .ok_or_else(|| anyhow::anyhow!("No choices returned from OpenRouter"))?;

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
