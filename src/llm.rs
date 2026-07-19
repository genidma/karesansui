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

/// Models that are FREE on OpenRouter. The gardener will ONLY ever call one of
/// these. Anything else (including paid models) is rejected at construction
/// time, so this binary can never accidentally spend credits.
const FREE_MODELS: &[&str] = &[
    "google/gemini-flash-8b:free",
    "meta-llama/llama-3.1-8b-instruct:free",
    "meta-llama/llama-3.2-3b-instruct:free",
    "deepseek/deepseek-chat-v3-0324:free",
    "qwen/qwen2.5-7b-instruct:free",
    "microsoft/phi-3-mini-128k-instruct:free",
];

/// Minimal OpenRouter client. Reads OPENROUTER_API_KEY from env (via dotenvy).
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

        // Enforce free-tier only. Fall back to the default free model if the
        // requested one isn't on the allowlist (and warn), rather than silently
        // allowing a paid model through.
        let requested = model.into();
        let model = if FREE_MODELS.contains(&requested.as_str()) {
            requested
        } else {
            eprintln!(
                "⚠ model '{requested}' is not on the free allowlist; using default free model instead"
            );
            "google/gemini-flash-8b:free".to_string()
        };

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            width,
            height,
        })
    }

    /// Ask the LLM for the next action given the current garden state.
    pub async fn next_action(&self, state: &str) -> Result<Action> {
        let system = format!(
            "You are the gardener of an ASCII zen garden of size {w}x{h}. \
             Decide the NEXT single action to slowly build a peaceful, balanced garden. \
             Return ONLY a JSON object with one of these shapes:\n\
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
            "response_format": { "type": "json_object" },
            "temperature": 0.8,
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

        let action: Action = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("failed to parse LLM action '{content}': {e}"))?;
        Ok(action)
    }
}
