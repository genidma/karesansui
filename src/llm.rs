use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::garden::Action;

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

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

const FREE_MODELS: &[&str] = &[
    "tencent/hy3:free",
    "google/gemma-4-31b-it:free",
    "google/gemma-4-26b-a4b-it:free",
    "poolside/laguna-xs-2.1:free",
    "openai/gpt-oss-20b:free",
    "cohere/north-mini-code:free",
];

pub struct Gardener {
    client: reqwest::Client,
    api_key: String,
    model: String,
    width: usize,
    height: usize,
}

impl Gardener {
    pub fn new(model: impl Into<String>, width: usize, height: usize) -> Result<Self> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| anyhow::anyhow!("OPENROUTER_API_KEY not set (add it to .env)"))?;

        let requested = model.into();
        let model = if FREE_MODELS.contains(&requested.as_str()) {
            requested
        } else {
            eprintln!(
                "⚠ model {requested} is not on the free allowlist; using default free model instead"
            );
            "tencent/hy3:free".to_string()
        };

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            width,
            height,
        })
    }

    pub async fn next_action(&self, state: &str) -> Result<Action> {
        let system = format!(
            "You are the gardener of an ASCII zen garden of size {w}x{h}. \
             Decide the NEXT single action to slowly build a peaceful, balanced garden. \
             Return ONLY valid raw JSON with one of these shapes (no markdown code blocks, no explanation):\n\
             {{\"action\":\"place_rock\",\"x\":<0-{xmax}>,\"y\":<0-{ymax}>,\"size\":<1-3>}}\n\
             {{\"action\":\"rake_line\",\"y\":<0-{ymax}>,\"x1\":<0-{xmax}>,\"x2\":<0-{xmax}>}}\n\
             {{\"action\":\"draw_border\"}}\n\
             {{\"action\":\"done\"}}\n\
             Prefer starting with draw_border, then rake_lines, then scatter a few rocks. \
             Call done once the garden feels complete (roughly 12-20 actions).",
            w = self.width,
            h = self.height,
            xmax = self.width - 1,
            ymax = self.height - 1,
        );

        let user = format!("Current garden:\n{state}\nWhat is your next action?");

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ],
            "temperature": 0.7,
        });

        let resp = self
            .client
            .post(OPENROUTER_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/karesansui")
            .header("X-Title", "karesansui")
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<ORResponse>()
            .await?;

        let content = resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("no choices returned from OpenRouter"))?;

        // Strip ```json markdown wrappers if the model includes them
        let clean = content
            .trim()
            .strip_prefix("```json")
            .unwrap_or(content.trim())
            .strip_prefix("```")
            .unwrap_or(content.trim())
            .strip_suffix("```")
            .unwrap_or(content.trim())
            .trim();

        let action: Action = serde_json::from_str(clean)
            .map_err(|e| anyhow::anyhow!("failed to parse LLM action {clean}: {e}"))?;
        Ok(action)
    }
}
