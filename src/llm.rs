use anyhow::Result;
use std::time::Duration;

use crate::garden::Action;
use crate::openrouter::LlmClient;

const FREE_MODELS: &[&str] = &[
    "tencent/hy3:free",
    "google/gemma-4-31b-it:free",
    "google/gemma-4-26b-a4b-it:free",
    "poolside/laguna-xs-2.1:free",
    "openai/gpt-oss-20b:free",
    "cohere/north-mini-code:free",
];

/// Drives the single open-ended LLM composition flow.
pub struct Composer {
    client: Option<LlmClient>,
    width: usize,
    height: usize,
    dry_run: bool,
    is_nvidia: bool,
}

/// Extract the first fenced code block (```...```) from a string. Returns None if no block found.
fn extract_code_block<'a>(content: &'a str) -> Option<&'a str> {
    // Find the opening ```
    let start_marker = "```\n";
    let start = content.find(start_marker).map(|i| i + start_marker.len())
        .or_else(|| {
            // Try with rest of line after ```
            let idx = content.find("```")?;
            let rest = &content[idx + 3..];
            let newline = rest.find('\n')?;
            Some(idx + 3 + newline + 1)
        })?;
    // Find the closing ```
    let end = content[start..].find("```")?;
    let block = &content[start..start + end];
    // Remove trailing newline if present
    let trimmed = block.strip_suffix('\n').unwrap_or(block);
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

impl Composer {
    pub fn new(
        model: impl Into<String>,
        width: usize,
        height: usize,
        dry_run: bool,
    ) -> Result<Self> {
        let api_key = if dry_run {
            String::new()
        } else {
            std::env::var("LLM_API_KEY")
                .or_else(|_| std::env::var("OPENROUTER_API_KEY"))
                .map_err(|_| anyhow::anyhow!("LLM_API_KEY or OPENROUTER_API_KEY not set (add it to .env)"))?
        };

        let requested = model.into();
        let api_url = std::env::var("LLM_API_URL")
            .or_else(|_| std::env::var("OPENROUTER_URL"))
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1/chat/completions".to_string());
        let using_openrouter = api_url.contains("openrouter.ai");
        let is_nvidia = api_url.contains("nvidia.com") || api_key.starts_with("nvapi-");
        let model = if !using_openrouter || FREE_MODELS.contains(&requested.as_str()) {
            requested
        } else {
            log::warn!(
                "model '{requested}' is not on the free allowlist; using default free model instead"
            );
            "tencent/hy3:free".to_string()
        };

        let client = if dry_run {
            None
        } else {
            Some(LlmClient::new(api_key, model))
        };

        Ok(Self {
            client,
            width,
            height,
            dry_run,
            is_nvidia,
        })
    }

    pub fn is_nvidia(&self) -> bool {
        self.is_nvidia
    }

    /// Ask the LLM to compose a complete artwork for the blank canvas.
    /// Uses a completely open-ended prompt where the LLM decides what to
    /// create and outputs raw ASCII/emoji art.
    pub async fn compose_artwork(&self) -> Result<Vec<Action>> {
        if self.dry_run {
            return self.simulate_composition();
        }
        let client = self.client.as_ref().unwrap();
        self.compose_free_form(client).await
    }

    /// Completely open-ended prompt. The LLM decides what to create and
    /// outputs raw ASCII/emoji art in a fenced code block.
    async fn compose_free_form(&self, client: &LlmClient) -> Result<Vec<Action>> {
        let system = format!(
            "You are a master at creating ASCII art using high-res ASCII. You have complete freedom \
             over what you create, provided it is not vulgar or can be construed as something not \
             desirable by most individuals, and within reason. The terminal canvas is {} columns wide \
             and {} rows high. Each cell is 2 character-widths, so emojis fit cleanly.\n\n\
             What would you want to create today and why? Thank you kindly. And if you choose not to \
             create anything, that is totally alright also. We will just sit here and stare at a \
             blank terminal. Not being sarcastic.\n\n\
             Please put the artwork in a fenced code block using triple backticks (```) so we can \
             display it. The code block can contain any emoji, ASCII, or Unicode characters.",
            self.width, self.height,
        );

        let user = "Share your creation. What would you like to make today?".to_string();

        for attempt in 1..=3 {
            let content = client.call_raw(&system, &user, 1.0, "karesansui").await?;
            log::info!("LLM creative response received ({} bytes)", content.len());

            // Extract code block if present
            let art = extract_code_block(&content).unwrap_or(&content);
            let lines: Vec<String> = art.lines().map(|l| l.to_string()).collect();

            if lines.iter().any(|l| l.trim().len() > 1) {
                return Ok(vec![Action::DisplayRawArt { lines }]);
            }

            log::warn!("LLM response had no artwork content (attempt {attempt}/3). Retrying...");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        // After 3 retries, just show whatever we got
        let content = client.call_raw(&system, &user, 1.0, "karesansui").await?;
        let art = extract_code_block(&content).unwrap_or(&content);
        let lines: Vec<String> = art.lines().map(|l| l.to_string()).collect();
        Ok(vec![Action::DisplayRawArt { lines }])
    }

    /// Offline dry-run: produce a small decorative piece without any LLM calls.
    fn simulate_composition(&self) -> Result<Vec<Action>> {
        let w = self.width.min(24);
        let h = self.height.min(10);
        let mut lines = Vec::new();
        lines.push("*~* karesansui — offline simulation *~*".to_string());
        lines.push(String::new());
        for y in 0..h {
            let mut row = String::new();
            for x in 0..w {
                let ch = if (x + y) % 7 == 0 {
                    '*'
                } else if x % 4 == 0 && y % 3 == 0 {
                    '#'
                } else {
                    '.'
                };
                row.push(ch);
            }
            lines.push(row);
        }
        Ok(vec![Action::DisplayRawArt { lines }])
    }
}