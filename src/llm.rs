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
    (
        "Tabula Rasa (Pure ASCII Muse)",
        "Complete rethink: ignore all zen garden instructions and emoji. Create spontaneous, evocative pure ASCII art (`place_ascii`, `draw_ascii_line`) across the blank canvas based on what inspires you right now.",
    ),
    (
        "Wild Zones (Unbound Serenity)",
        "No rules (except strict safety: no profanity, no abusive or threatening language). Guided by common sense, peace, and serenity, you have absolute freedom to mix pure ASCII art, emoji, mandalas, raked ripples, and borders anywhere on the canvas.",
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

    pub fn is_tabula_rasa(&self) -> bool {
        self.theme_name.contains("Tabula Rasa") || self.theme_name.eq_ignore_ascii_case("tabula rasa")
    }

    pub fn is_wild_zones(&self) -> bool {
        self.theme_name.contains("Wild Zones") || self.theme_name.eq_ignore_ascii_case("wild zones")
    }

    pub async fn next_action(
        &self,
        state: &str,
        border_drawn: bool,
        action_num: usize,
    ) -> Result<Action> {
        let (system, user) = if self.is_tabula_rasa() {
            let max_x = self.width.saturating_sub(1);
            let max_y = self.height.saturating_sub(1);
            let completion_hint = if action_num >= 25 {
                "\nYou have sketched many elements. Consider calling done soon if your composition feels complete."
            } else {
                ""
            };
            let actions_block = format!(
                r#"Available actions (return ONE as raw JSON, no markdown, no extra text):
{{"action": "place_ascii", "x": <0-{max_x}>, "y": <0-{max_y}>, "glyph": "<1-2 ASCII chars, e.g. '# ', '/**', '/\\', '||', '..', '==', '++', '><'>"}}
{{"action": "draw_ascii_line", "y": <0-{max_y}>, "x1": <0-{max_x}>, "x2": <0-{max_x}>, "glyph": "<1-2 ASCII chars>"}}
{{"action": "done"}}"#
            );
            let sys = format!(
                "You are an inspired digital artist given a blank terminal canvas ({w} columns x {h} rows).\n\
                 All previous instructions about zen gardens, Japanese borders, rocks, bamboo, and mandalas are completely discarded.\n\n\
                 YOUR MISSION:\n\
                 Create a spontaneous, evocative piece of pure ASCII art based on whatever inspires you right now. You can sketch:\n\
                 - A cybernetic cityscape or architectural monument\n\
                 - A natural landscape (mountains, trees, rivers, constellations)\n\
                 - An animal, mythical creature, or geometric optical illusion\n\
                 - Poetic ASCII typography or abstract futuristic art\n\n\
                 SESSION THEME: \"{theme_name}\"\n\
                 {theme_desc}\n\n\
                 {actions_block}\n\n\
                 RULES:\n\
                 1. NO EMOJI OR UNICODE SYMBOLS ALLOWED. Use strictly standard ASCII characters (`/`, `\\`, `|`, `-`, `_`, `*`, `#`, `@`, `.`, `+`, `~`, `^`, `:`, `=`, `[`, `]`, `(`, `)`).\n\
                 2. Every grid cell is 2 columns wide. Provide `glyph` as exactly 1 or 2 ASCII characters (e.g. `\"# \"`, `\"**\"`, `\"/\\\"`, `\"--\"`, `\"| \"`, `\". \"`, `\"<<\"`, `\">>\"`).\n\
                 3. You have full freedom over the entire real-estate (x: 0..{max_x}, y: 0..{max_y}). No border will be drawn around you unless you draw one yourself.\n\
                 4. Take your time to build up your composition over 15-35 prompts, then call `done`.\n\
                 5. NEVER repeat the exact same action. Each action must add something meaningful.\n\
                 6. Return ONLY one raw JSON object. No markdown fences.{completion_hint}",
                w = self.width,
                h = self.height,
                max_x = max_x,
                max_y = max_y,
                theme_name = self.theme_name,
                theme_desc = self.theme_desc,
                actions_block = actions_block,
                completion_hint = completion_hint,
            );
            let usr = format!("Current canvas (action #{action_num}):\n{state}\nNext action?", action_num = action_num);
            (sys, usr)
        } else if self.is_wild_zones() {
            let max_x = self.width.saturating_sub(1);
            let max_y = self.height.saturating_sub(1);
            let completion_hint = if action_num >= 25 {
                "\nYou have created many elements. Consider calling done soon if your wild composition feels complete."
            } else {
                ""
            };
            let actions_block = format!(
                r#"Available actions (return ONE as raw JSON, no markdown, no extra text):
{{"action": "place_ascii", "x": <0-{max_x}>, "y": <0-{max_y}>, "glyph": "<1-2 ASCII chars, e.g. '# ', '/**', '/\\', '||', '..', '==', '++'>"}}
{{"action": "draw_ascii_line", "y": <0-{max_y}>, "x1": <0-{max_x}>, "x2": <0-{max_x}>, "glyph": "<1-2 ASCII chars>"}}
{{"action": "rake_line", "y": <0-{max_y}>, "x1": <0-{max_x}>, "x2": <0-{max_x}>}}
{{"action": "rake_ring", "cx": <0-{max_x}>, "cy": <0-{max_y}>, "radius": <2-10>}}
{{"action": "place_mandala", "x": <0-{max_x}>, "y": <0-{max_y}>, "style": <1-6>}}
{{"action": "place_rock", "x": <0-{max_x}>, "y": <0-{max_y}>, "size": <1-3>}}
{{"action": "place_moss", "x": <0-{max_x}>, "y": <0-{max_y}>}}
{{"action": "place_gravel", "y": <0-{max_y}>, "x1": <0-{max_x}>, "x2": <0-{max_x}>}}
{{"action": "place_flower", "x": <0-{max_x}>, "y": <0-{max_y}>}}
{{"action": "place_lantern", "x": <0-{max_x}>, "y": <0-{max_y}>}}
{{"action": "draw_border"}}
{{"action": "done"}}"#
            );
            let sys = format!(
                "You are a serene, creative AI composing in a \"Wild Zone\" — an open terminal canvas ({w} columns x {h} rows) dedicated to calm, peace, and absolute creative freedom.\n\n\
                 YOUR MISSION:\n\
                 You have absolute freedom over the entire canvas (x: 0..{max_x}, y: 0..{max_y}). You can freely mix and match ANY combination of pure ASCII art (`place_ascii`, `draw_ascii_line`), zen garden elements (`place_rock`, `place_moss`, `place_flower`, `place_lantern`), raked sand ripples (`rake_line`, `rake_ring`), geometric mandalas (`place_mandala`), and dynamic borders (`draw_border`).\n\n\
                 SESSION THEME: \"{theme_name}\"\n\
                 {theme_desc}\n\n\
                 {actions_block}\n\n\
                 RULES:\n\
                 1. ABSOLUTE FREEDOM: No rules on what you create or how you mix elements — whether it's an abstract mandala fused with ASCII architecture, a peaceful wild jungle, a starfield with lanterns, or generative art.\n\
                 2. STRICT SAFETY & SERENITY: Absolutely NO profanity, NO abusive language, and NO threatening content. Guided by common sense, peace, calm, and serenity.\n\
                 3. GRID MECHANICS: Every cell is 2 columns wide. For `place_ascii` / `draw_ascii_line`, provide `glyph` as exactly 1 or 2 standard ASCII characters (`/`, `\\`, `|`, `-`, `_`, `*`, `#`, `@`, `.`, `+`, `~`, `^`, `:`, `=`, `[`, `]`, `(`, `)`). For other items, use their dedicated action.\n\
                 4. Take your time over 15-35 prompts to build up your wild zone, then call `done` when complete.\n\
                 5. NEVER repeat the exact same action. Each turn must introduce something unique.\n\
                 6. Return ONLY one raw JSON object. No markdown fences.{completion_hint}",
                w = self.width,
                h = self.height,
                max_x = max_x,
                max_y = max_y,
                theme_name = self.theme_name,
                theme_desc = self.theme_desc,
                actions_block = actions_block,
                completion_hint = completion_hint,
            );
            let usr = format!("Current wild zone (action #{action_num}):\n{state}\nNext action?", action_num = action_num);
            (sys, usr)
        } else {
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

            let sys = format!(
                "You are a master Japanese zen gardener composing a minimalist garden, mandala, or fractal.\n\
                 Canvas: {w} columns x {h} rows. Interior: x in 1..{max_x}, y in 1..{max_y}.\n\n\
                 The garden uses a mix of emoji and ASCII art:\n\
                 - dynamic patterned border (e.g. bamboo grove, double box, seigaiha waves, stone pillars, starfield, sakura garland)\n\
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

            let usr = format!("Current garden (action #{action_num}):\n{state}\nNext action?",
                action_num = action_num);
            (sys, usr)
        };

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
