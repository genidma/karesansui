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

pub const THEMES: &[(&str, &str)] = &[
    (
        "Moonlit Reef",
        "A nocturnal ocean scene. Place rock clusters as coral reefs \
         surrounded by sweeping raked sand curves. Use moss as sea-foam \
         and flowers as bioluminescent blooms.",
    ),
    (
        "Dragon Tail Ripples",
        "Long flowing rake lines sweep across the garden diagonally, \
         like the wake of a dragon. Rocks form a loose S-curve. \
         Lanterns mark the dragon's resting spots.",
    ),
    (
        "Three Mountain Sanzen",
        "Classic three-stone triadic composition. Place three prominent \
         rock groups: one large central stone flanked by two smaller groups. \
         Rake concentric sand ripples around each. Moss at the bases.",
    ),
    (
        "Autumn Sand Drift",
        "Wind-blown patterns. Rake lines flow left to right with varying \
         lengths. Scatter flowers like fallen cherry petals. Small rocks \
         dot the landscape. Gravel patches like dry streambeds.",
    ),
    (
        "Island Archipelago",
        "Multiple isolated rock clusters as islands with moss growing on them. \
         Raked sand flows between islands like ocean currents. \
         Gravel as shallow shores. Lanterns guide the way.",
    ),
    (
        "Stepping Stone Path",
        "A diagonal path of evenly-spaced rocks from one corner toward the \
         opposite. Raked sand flows perpendicular to the path. Moss along \
         path edges. Lanterns at the start and end.",
    ),
    (
        "Crane and Turtle",
        "Two distinct rock groupings: one vertical (crane) and one wide \
         (turtle). Raked sand circles around both. Gravel connects them. \
         Flowers accent the turtle's shell.",
    ),
    (
        "Zen Minimalist",
        "Extreme restraint. Only 2-3 rocks placed with precision. \
         Rake most interior rows fully for uniform sand texture. \
         One small moss accent. One lantern in a corner.",
    ),
    (
        "Forest Clearing",
        "Dense moss patches along top and bottom edges like canopy shadows. \
         A central clearing of raked sand with a prominent rock. \
         Flowers bloom at the edge of the tree line.",
    ),
    (
        "Whirlpool Basin",
        "Raked lines of varying length converge on a central rock cluster. \
         Shorter rakes near center, longer at edges. Gravel marks the outer \
         rim. A lantern watches over the basin.",
    ),
    (
        "Scattered Stars",
        "Many small rocks scattered like a star field. A few larger rocks as \
         constellations. Flowers as distant nebulae. Minimal raking. \
         A lone lantern as the moon.",
    ),
    (
        "River Delta",
        "Raked sand lines fan from one side like a branching river. \
         Rocks as riverbed stones. Moss at the banks. Gravel in shallow \
         areas. Flowers bloom along the water's edge.",
    ),
    // Mandala & Fractal Themes
    (
        "Sacred Geometry Mandala",
        "Center around a focal mandala symbol (`place_mandala` style 2 or 3). \
         Surround with concentric circular sand rings (`rake_ring`). \
         Use geometric diamond and star symbols (`◈ `, `✦ `) with radial symmetry and deep minimalism.",
    ),
    (
        "Enso Fractal Solitude",
        "Extreme minimalist void anchored by a single Enso circle (`⭕`, `place_mandala` style 1). \
         Radiate concentric rings (`rake_ring`) and tiny fractal stars (`✦ `) outward like echoes in an infinite void.",
    ),
    (
        "Concentric Rings of Sanzen",
        "Multiple overlapping or nested circular rings (`rake_ring`) around 2 or 3 carefully placed stones (`🪨`, `🗿`). \
         Minimalist geometric balance between circular geometry and straight horizontal rakes (`~~`).",
    ),
    (
        "Fractal Starfield Void",
        "A minimalist fractal arrangement of stars (`✦ `) and geometric crests (`❖ `). \
         Use circular rings (`rake_ring`) and gravel patches (`··`) around each node to create a self-similar, hypnotic lattice.",
    ),
    (
        "Yin-Yang Balance",
        "Strict duality and equilibrium. Place `☯ ` (`place_mandala` style 5) as the central anchor. \
         Surround one side with flowing circular raked sand (`rake_ring`), while the other side holds textured gravel (`··`) and moss (`🌿`).",
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
    pub fn new(
        model: impl Into<String>,
        width: usize,
        height: usize,
        theme_choice: Option<&str>,
    ) -> Result<Self> {
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
        let chosen_theme = match theme_choice {
            Some(choice) if !choice.trim().is_empty() && !choice.eq_ignore_ascii_case("random") && choice != "0" => {
                if let Ok(num) = choice.trim().parse::<usize>() {
                    if num >= 1 && num <= THEMES.len() {
                        THEMES[num - 1]
                    } else {
                        eprintln!("⚠ theme index {num} out of bounds (1..{}); choosing randomly", THEMES.len());
                        *THEMES.choose(&mut rng).unwrap()
                    }
                } else {
                    // Try case-insensitive substring match across theme names
                    THEMES
                        .iter()
                        .find(|(name, _)| name.to_lowercase().contains(&choice.to_lowercase()))
                        .copied()
                        .unwrap_or_else(|| {
                            eprintln!("⚠ theme '{choice}' not found; choosing randomly");
                            *THEMES.choose(&mut rng).unwrap()
                        })
                }
            }
            _ => *THEMES.choose(&mut rng).unwrap(),
        };

        let (name, desc) = chosen_theme;

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

    pub async fn next_action(
        &self,
        state: &str,
        border_drawn: bool,
        action_num: usize,
    ) -> Result<Action> {
        let max_x = self.width.saturating_sub(2);
        let max_y = self.height.saturating_sub(2);

        let actions_block = if border_drawn {
            format!(
                r#"Available actions (return ONE as raw JSON, no markdown, no extra text):
{{"action": "rake_line", "y": <1-{max_y}>, "x1": <1-{max_x}>, "x2": <1-{max_x}>}}
{{"action": "rake_ring", "cx": <1-{max_x}>, "cy": <1-{max_y}>, "radius": <2-10>}}
{{"action": "place_mandala", "x": <1-{max_x}>, "y": <1-{max_y}>, "style": <1-6>}}
{{"action": "place_rock", "x": <1-{max_x}>, "y": <1-{max_y}>, "size": <1-3>}}
{{"action": "place_moss", "x": <1-{max_x}>, "y": <1-{max_y}>}}
{{"action": "place_gravel", "y": <1-{max_y}>, "x1": <1-{max_x}>, "x2": <1-{max_x}>}}
{{"action": "place_flower", "x": <1-{max_x}>, "y": <1-{max_y}>}}
{{"action": "place_lantern", "x": <1-{max_x}>, "y": <1-{max_y}>}}
{{"action": "done"}}"#,
                max_x = max_x, max_y = max_y,
            )
        } else {
            String::from(
                r#"The garden has no border yet. Your first action MUST be:
{"action": "draw_border"}"#,
            )
        };

        let completion_hint = if action_num >= 20 {
            "\nYou have placed many elements. Consider calling done soon if it looks complete."
        } else {
            ""
        };

        let system = format!(
            "You are a master Japanese zen gardener composing a minimalist garden, mandala, or fractal.\n\
             Canvas: {w} columns x {h} rows. Interior: x in 1..{max_x}, y in 1..{max_y}.\n\n\
             The garden uses a mix of emoji and ASCII art:\n\
             - 🎋 bamboo border\n\
             - ~~ raked horizontal sand ripples, ◎  concentric ring ripples (`rake_ring`)\n\
             - 🪨 small rock, 🗿 large rock\n\
             - 🌿 moss, 🌸 cherry blossom, 🏮 stone lantern, ·· gravel path\n\
             - Minimalist Mandala / Fractal styles (`place_mandala` style 1-6): ⭕ Enso, ◎  concentric, ◈  diamond, ✦  star, ☯  yin-yang, ❖  crest\n\n\
             SESSION THEME: \"{theme_name}\"\n\
             {theme_desc}\n\n\
             {actions_block}\n\n\
             RULES:\n\
             1. Use the FULL canvas. Spread actions cleanly with geometric precision and restraint.\n\
             2. For mandala themes, use `place_mandala` and `rake_ring` to build concentric circular patterns.\n\
             3. Rocks: size 1 (🪨), size 2 (🗿), size 3 (🗿). Group or scatter cleanly.\n\
             4. Moss 🌿 near stones. Flowers 🌸 and Lanterns 🏮 as focal accents.\n\
             5. Aim for 15-25 total actions, maintaining clean space, then call done.\n\
             6. NEVER repeat the same exact action. Each must be DIFFERENT.\n\
             7. Return ONLY one raw JSON object. No markdown fences.{completion_hint}",
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
