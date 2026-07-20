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
        let max_x = self.width.saturating_sub(2);
        let max_y = self.height.saturating_sub(2);

        let system = format!(
            "You are the gardener of an ASCII zen garden of size {w}x{h}.\n\
             Decide the NEXT single action to build a peaceful, balanced zen garden.\n\
             Return ONLY valid raw JSON matching one of these shapes (no markdown code blocks, no extra text):\n\
             {{\"action\":\"place_rock\",\"x\":<1-{max_x}>,\"y\":<1-{max_y}>,\"size\":<1-3>}}\n\
             {{\"action\":\"rake_line\",\"y\":<1-{max_y}>,\"x1\":<1-{max_x}>,\"x2\":<1-{max_x}>}}\n\
             {{\"action\":\"draw_border\"}}\n\
             {{\"action\":\"done\"}}\n\n\
             RULES:\n\
             1. If the outer border `#` is missing, call `draw_border` first.\n\
             2. If the outer border `#` is ALREADY drawn, DO NOT call `draw_border` again!\n\
             3. Fill empty rows inside the border with raked sand lines (`rake_line`).\n\
             4. Scatter several rocks (`place_rock`) of various sizes inside the garden.\n\
             5. Do NOT call `done` until the garden has multiple raked lines and several rocks placed.",
            w = self.width,
            h = self.height,
            max_x = max_x,
            max_y = max_y,
        );

        let user = format!("Current garden state:\n{state}\nWhat is your next action?");

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
