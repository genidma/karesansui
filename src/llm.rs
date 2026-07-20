use anyhow::Result;
use rand::seq::IndexedRandom;
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

const THEMES: &[(&str, &str)] = &[
    (
        "Moonlit Reef",
        "A nocturnal ocean scene. Place large central rock clusters as coral reefs \
         surrounded by sweeping, wide raked sand curves. Leave calm open pools of \
         empty space near the edges. Use moss accents sparingly as sea-foam.",
    ),
    (
        "Dragon Tail Ripples",
        "Long, flowing rake lines that sweep across most of the garden diagonally, \
         like the wake of a dragon's tail. Rocks are placed in a loose S-curve. \
         Gravel patches mark where the dragon rested.",
    ),
    (
        "Three Mountain Sanzen",
        "Classic three-stone triadic composition (sanzon-seki). Place three prominent \
         rock groups: one large central stone flanked by two smaller groups at \
         asymmetric distances. Rake concentric sand ripples around each group.",
    ),
    (
        "Autumn Sand Drift",
        "Asymmetric gravel patches like wind-blown autumn leaves scattered across \
         the sand. Small rocks dot the landscape. Rake lines flow from left to right \
         with varying lengths, like wind patterns.",
    ),
    (
        "Island Archipelago",
        "Multiple isolated rock clusters as islands. Each island has 2-3 rocks of \
         varying size with moss growing on them. Raked sand flows between islands \
         like ocean currents. Use gravel as shallow shores.",
    ),
    (
        "Stepping Stone Path",
        "A diagonal path of evenly-spaced single rocks from one corner toward the \
         opposite. Raked sand flows perpendicular to the path. Moss grows along \
         the path edges. Empty calm zones on either side.",
    ),
    (
        "Crane and Turtle",
        "Two distinct rock groupings: one tall vertical arrangement (the crane) \
         and one low wide arrangement (the turtle). Raked sand circles around both. \
         Gravel connects them like a bridge.",
    ),
    (
        "Zen Minimalist",
        "Extreme restraint. Only 2-3 rocks placed with mathematical precision. \
         Rake every interior row fully from edge to edge for uniform sand texture. \
         Leave one small moss accent near a rock.",
    ),
    (
        "Forest Clearing",
        "Dense moss patches along the top and bottom edges like tree canopy shadows. \
         A central clearing of raked sand with a single prominent rock. Gravel \
         paths lead inward from the sides.",
    ),
    (
        "Whirlpool Basin",
        "Raked lines of varying length create a spiral-like pattern converging on a \
         central rock cluster. Shorter rake lines near the center, longer at the \
         edges. Gravel marks the outer rim.",
    ),
    (
        "Scattered Stars",
        "Many small rocks (size 1) placed across the garden like a star field. \
         A few medium rocks as constellations. Minimal raking; mostly open \
         empty space with small gravel patches.",
    ),
    (
        "River Delta",
        "Raked sand lines fan out from one side to the other like a river branching \
         into a delta. Rocks are placed as riverbed stones. Moss grows at the \
         river banks. Gravel fills shallow areas.",
    ),
];

pub struct Gardener {
    client: reqwest::Client,
    api_key: String,
    model: String,
    width: usize,
    height: usize,
    theme_name: String,
    theme_desc: String,
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

        let mut rng = rand::rng();
        let (name, desc) = THEMES.choose(&mut rng).unwrap();

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            width,
            height,
            theme_name: name.to_string(),
            theme_desc: desc.to_string(),
        })
    }

    pub fn theme_name(&self) -> &str {
        &self.theme_name
    }

    /// Ask the LLM for the next action. `border_drawn` and `action_num` let
    /// us adjust the prompt so the LLM doesn't get stuck repeating draw_border.
    pub async fn next_action(
        &self,
        state: &str,
        border_drawn: bool,
        action_num: usize,
    ) -> Result<Action> {
        let max_x = self.width.saturating_sub(2);
        let max_y = self.height.saturating_sub(2);

        // Build the available-actions block dynamically.
        let actions_block = if border_drawn {
            format!(
                r#"Available actions (return ONE as raw JSON, no markdown, no commentary):
{{"action": "rake_line", "y": <1-{max_y}>, "x1": <1-{max_x}>, "x2": <1-{max_x}>}}
{{"action": "place_rock", "x": <1-{max_x}>, "y": <1-{max_y}>, "size": <1-3>}}
{{"action": "place_moss", "x": <1-{max_x}>, "y": <1-{max_y}>}}
{{"action": "place_gravel", "y": <1-{max_y}>, "x1": <1-{max_x}>, "x2": <1-{max_x}>}}
{{"action": "done"}}"#,
                max_x = max_x, max_y = max_y,
            )
        } else {
            String::from(
                r#"The garden has no border yet. Your first action MUST be:
{"action": "draw_border"}"#,
            )
        };

        // Nudge toward completion when we've done enough actions.
        let completion_hint = if action_num >= 20 {
            "\nYou have placed many elements. Consider calling done soon if it looks complete."
        } else {
            ""
        };

        let system = format!(
            "You are a master zen gardener composing a unique ASCII zen garden.\n\
             Canvas: {w} columns x {h} rows. Interior: x in 1..{max_x}, y in 1..{max_y}.\n\n\
             SESSION THEME: \"{theme_name}\"\n\
             {theme_desc}\n\n\
             {actions_block}\n\n\
             RULES:\n\
             1. Use the FULL canvas. Spread actions across many different rows and columns.\n\
             2. Vary rake_line lengths: some span the full row, others are short segments.\n\
             3. Rocks: size 1 gives o, size 2 gives O, size 3 gives @. Group or scatter per theme.\n\
             4. Moss (*) goes near rocks or edges. Gravel (.) fills paths or shores.\n\
             5. Aim for 15-25 total actions, then call done.\n\
             6. NEVER repeat an action you already did. Each action should be DIFFERENT.\n\
             7. Return ONLY one raw JSON object. No markdown fences, no explanation.{completion_hint}",
            w = self.width,
            h = self.height,
            max_x = max_x,
            max_y = max_y,
            theme_name = self.theme_name,
            theme_desc = self.theme_desc,
            actions_block = actions_block,
            completion_hint = completion_hint,
        );

        let user = format!("Current garden (action #{action_num}):\n{state}\nNext action?",
            action_num = action_num);

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ],
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
