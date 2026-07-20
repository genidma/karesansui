use anyhow::Result;
use serde::Deserialize;
use serde_json::json;

use crate::garden::Action;

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

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
                "⚠ model '{requested}' is not on the free allowlist; using default free model instead"
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

        let shapes = r#"{"action": "place_rock", "x": 10, "y": 5, "size": 2}
{"action": "rake_line", "y": 4, "x1": 2, "x2": 44}
{"action": "draw_border"}
{"action": "done"}"#;

        let system = format!(
            "You are a master zen gardener crafting a unique, artistic ASCII zen garden inside a {w}x{h} grid.
             Decide the NEXT single action to compose a serene and visually intriguing landscape.
             Return ONLY valid raw JSON matching one of these shapes (no markdown code blocks, no extra text):
{shapes}

             MASTER GARDENER CREATIVE RULES:
             1. BORDER: If the outer border `#` is missing, start with `draw_border`. Once drawn, DO NOT call `draw_border` again!
             2. REAL ESTATE: Utilize the entire available canvas (rows 1 to {max_y}, columns 1 to {max_x}). Do not leave large empty gaps.
             3. UNIQUE ZEN PATTERNS: Rake expressive sand ripples (`~`) across rows. Vary the start (`x1`) and end (`x2`) columns to create staggered, flowing wave patterns, islands of calm, or asymmetrical paths.
             4. ROCK COMPOSITION: Place rocks (`o`, `O`, `@`) thoughtfully using principles of Japanese rock gardening (Sanzen/triad groupings, central stone, island reefs, or focal stepping stones).
             5. COMPLETION: Keep building until the garden feels rich, balanced, and aesthetically unique (roughly 12-25 actions), then call `done`.",
            w = self.width,
            h = self.height,
            max_x = max_x,
            max_y = max_y,
            shapes = shapes,
        );

        let user = format!("Current garden state:
{state}
What is your next action?");

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ],
            "temperature": 0.75,
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
            .map_err(|e| anyhow::anyhow!("failed to parse LLM action '{clean}': {e}"))?;
        Ok(action)
    }
}
