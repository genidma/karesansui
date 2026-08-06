use anyhow::Result;
use rand::seq::IndexedRandom;
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

pub const THEMES: &[(&str, &str)] = &[
    (
        "Creative Freedom",
        "Unlimited creative ASCII art across an open terminal canvas. \
         No borders, no garden rules, no constraints — pure creative \
         expression with any emoji or ASCII characters using all available drawing actions.",
    ),
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
        "True liberation: all zen garden rules, raked sand, mandalas, and rigid borders are completely removed. Guided strictly by peace, calm, and serenity (zero profanity/threats/abuse), you have absolute freedom across the open canvas (`place_glyph`, `draw_line`, `draw_ring`, `fill_box`, `clear_cell`) using any emoji or ASCII characters.",
    ),
    (
        "Gridwright (The Deliberate Grid as Craft)",
        "Pixel-perfect grid art where every cell is a deliberate choice. Leverages exact coordinate geometry (`vec`), chunky 8-color palettes (`color`), and hard-edge block glyph rendering (`canvas`). Commands include `clear_canvas`, `fill_rectangle`, `draw_rectangle`, `fill_circle`, `draw_circle`, `draw_line_h`, `draw_line_v`, `draw_line_diag`, `draw_path`, and `set_pixel` with 0-indexed palette colors (`color_index: 0..7`) and 2-character wide block glyphs (`██`, `▓▓`, `▒▒`, `░░`) for layered, square-proportional pixel masterpieces.",
    ),
];

pub struct Gardener {
    client: Option<LlmClient>,
    width: usize,
    height: usize,
    theme_name: String,
    theme_desc: String,
    dry_run: bool,
    is_nvidia: bool,
}

fn strip_markdown_fence(s: &str) -> String {
    let s = s.trim();
    if s.starts_with("```") {
        let body = s.trim_start_matches(|c| c == '`' || c == '\n' || c == '\r');
        body.trim_end_matches('`').trim().to_string()
    } else {
        s.to_string()
    }
}

fn repair_json(s: &str) -> String {
    // Fix common LLM mistake: `"x":40"` → `"x":40` (stray quote after number)
    let bytes = s.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'"' && i + 3 < len {
            // Check if preceding chars end with `:DIGITS` (no opening quote)
            let mut j = i;
            while j > 0 && (bytes[j - 1] as char).is_ascii_digit() {
                j -= 1;
            }
            if j > 0 && bytes[j - 1] == b':' {
                let mut k = j;
                while k < i && (bytes[k] as char).is_ascii_digit() {
                    k += 1;
                }
                if k == i {
                    // This `"` is a stray quote after a number value — skip it
                    i += 1;
                    continue;
                }
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(result).unwrap_or_else(|_| s.to_string())
}

fn parse_action_batch(content: &str) -> Result<Vec<Action>> {
    for _ in 0..3 {
        let cleaned = strip_markdown_fence(content);
        if cleaned.starts_with('[') {
            if let Ok(actions) = serde_json::from_str::<Vec<Action>>(&cleaned) {
                return Ok(actions);
            }
        } else if let Ok(action) = serde_json::from_str::<Action>(&cleaned) {
            return Ok(vec![action]);
        }
        // Repair common LLM JSON errors and retry
        let repaired = repair_json(content);
        if repaired != content {
            if repaired.starts_with('[') {
                if let Ok(actions) = serde_json::from_str::<Vec<Action>>(&repaired) {
                    return Ok(actions);
                }
            }
        }
        break;
    }
    Err(anyhow::anyhow!("failed to parse LLM response as action array or single action: {content}"))
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

impl Gardener {
    pub fn new(
        model: impl Into<String>,
        width: usize,
        height: usize,
        theme_choice: Option<&str>,
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

        let mut rng = rand::rng();
        let chosen_theme = match theme_choice {
            Some(choice) if !choice.trim().is_empty() && !choice.eq_ignore_ascii_case("random") && choice != "0" => {
                if let Ok(num) = choice.trim().parse::<usize>() {
                    if num >= 1 && num <= THEMES.len() {
                        THEMES[num - 1]
                    } else {
                        log::warn!("theme index {num} out of bounds (1..{}); choosing randomly", THEMES.len());
                        *THEMES.choose(&mut rng).unwrap()
                    }
                } else {
                    if choice.eq_ignore_ascii_case("classic") {
                        *THEMES.choose(&mut rng).unwrap()
                    } else {
                        THEMES
                            .iter()
                            .find(|(name, _)| name.to_lowercase().contains(&choice.to_lowercase()))
                            .copied()
                            .unwrap_or_else(|| {
                                log::warn!("theme '{choice}' not found; using Creative Freedom");
                                THEMES[0]
                            })
                    }
                }
            }
            _ => THEMES[0],
        };

        let (name, desc) = chosen_theme;

        let client = if dry_run {
            None
        } else {
            Some(LlmClient::new(api_key, model))
        };

        Ok(Self {
            client,
            width,
            height,
            theme_name: name.to_string(),
            theme_desc: desc.to_string(),
            dry_run,
            is_nvidia,
        })
    }

    pub fn theme_name(&self) -> &str {
        &self.theme_name
    }

    pub fn is_creative_freedom(&self) -> bool {
        self.theme_name.eq_ignore_ascii_case("creative freedom")
    }

    pub fn is_tabula_rasa(&self) -> bool {
        self.theme_name.contains("Tabula Rasa") || self.theme_name.eq_ignore_ascii_case("tabula rasa")
    }

    pub fn is_wild_zones(&self) -> bool {
        self.theme_name.contains("Wild Zones") || self.theme_name.eq_ignore_ascii_case("wild zones")
    }

    pub fn is_nvidia(&self) -> bool {
        self.is_nvidia
    }

    /// Send a single comprehensive prompt to the LLM asking for a complete artwork.
    /// For Creative Freedom mode, uses a completely open-ended prompt where the LLM
    /// decides what to create and outputs raw ASCII art. For other themes, uses the
    /// structured action-based JSON prompt.
    pub async fn compose_artwork(&self, state: &str) -> Result<Vec<Action>> {
        if self.dry_run {
            return self.simulate_composition();
        }

        let client = self.client.as_ref().unwrap();

        if self.is_creative_freedom() {
            return self.compose_free_form(client).await;
        }

        let (sys_base, usr) = self.build_composition_prompt(state);
        let mut last_err = String::new();

        for attempt in 1..=3 {
            let system = if attempt == 1 {
                sys_base.clone()
            } else {
                format!(
                    "{sys_base}\n\nIMPORTANT: Your previous response had a JSON error: {last_err}\n\
                     Return ONLY valid raw JSON. No markdown fences. \
                     Make sure numbers are NOT quoted: use `\"x\": 40` NOT `\"x\": \"40\"`."
                )
            };
            let content = client.call_raw(&system, &usr, 1.0, "karesansui").await?;
            match parse_action_batch(&content) {
                Ok(actions) => return Ok(actions),
                Err(e) => {
                    last_err = format!("{e}");
                    log::warn!("LLM JSON parse failed (attempt {attempt}/3): {e}. Retrying...");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }

        Err(anyhow::anyhow!("LLM failed to produce valid JSON after 3 attempts. Last error: {last_err}"))
    }

    /// Creative Freedom v2: completely open-ended prompt. The LLM decides what to create
    /// and outputs raw ASCII/emoji art. Returns DisplayRawArt action with the extracted art.
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

    fn build_composition_prompt(&self, state: &str) -> (String, String) {
        if self.is_creative_freedom() {
            unreachable!("Creative Freedom uses compose_free_form, not build_composition_prompt")
        } else if self.is_tabula_rasa() {
            self.build_tabula_composition(state)
        } else if self.is_wild_zones() {
            self.build_wild_composition(state)
        } else {
            self.build_classic_composition(state)
        }
    }

    fn build_tabula_composition(&self, state: &str) -> (String, String) {
        let max_x = self.width.saturating_sub(1);
        let max_y = self.height.saturating_sub(1);

        let actions_list = format!(
            r#"Available actions (use these to build your composition):
{{"action": "place_ascii", "x": <0-{max_x}>, "y": <0-{max_y}>, "glyph": "<1-2 ASCII chars>"}}
{{"action": "draw_ascii_line", "y": <0-{max_y}>, "x1": <0-{max_x}>, "x2": <0-{max_x}>, "glyph": "<1-2 ASCII chars>"}}
{{"action": "done"}}"#
        );

        let sys = format!(
            "You are an inspired digital artist creating a complete pure ASCII artwork on a blank terminal canvas ({w} columns x {h} rows).\
             \n\nYOUR MISSION:\n\
             Create a spontaneous, evocative piece of pure ASCII art. No emoji — standard ASCII only.\
             \n\nSESSION THEME: \"{theme_name}\"\n\
             {theme_desc}\n\n\
             {actions_list}\n\n\
             INSTRUCTIONS:\n\
             1. Return a JSON array of 20-40 actions forming a complete piece.\n\
             2. Use strictly standard ASCII chars (`/`, `\\`, `|`, `-`, `_`, `*`, `#`, `@`, `.`, `+`, `~`, `:`, `=`, `(`, `)`).\n\
             3. End with {{\"action\": \"done\"}}.\n\
             4. Return ONLY a raw JSON array. No markdown fences.\n\n\
             Example:\n\
             [{{\"action\": \"place_ascii\", \"x\": 10, \"y\": 5, \"glyph\": \"# \"}}, {{\"action\": \"done\"}}]",
            w = self.width, h = self.height,
            theme_name = self.theme_name, theme_desc = self.theme_desc,
            actions_list = actions_list,
        );

        let usr = format!(
            "Current canvas (blank):\n{state}\n\nCreate your complete composition as a JSON array:"
        );

        (sys, usr)
    }

    fn build_wild_composition(&self, state: &str) -> (String, String) {
        let max_x = self.width.saturating_sub(1);
        let max_y = self.height.saturating_sub(1);

        let actions_list = format!(
            r#"Available actions (use these to build your composition):
{{"action": "place_glyph", "x": <0-{max_x}>, "y": <0-{max_y}>, "glyph": "<any single emoji or 1-2 ASCII chars>"}}
{{"action": "draw_line", "y": <0-{max_y}>, "x1": <0-{max_x}>, "x2": <0-{max_x}>, "glyph": "<glyph>"}}
{{"action": "draw_ring", "cx": <0-{max_x}>, "cy": <0-{max_y}>, "radius": <2-12>, "glyph": "<glyph>"}}
{{"action": "fill_box", "x1": <0-{max_x}>, "y1": <0-{max_y}>, "x2": <0-{max_x}>, "y2": <0-{max_y}>, "glyph": "<glyph>"}}
{{"action": "clear_cell", "x": <0-{max_x}>, "y": <0-{max_y}>}}
{{"action": "done"}}"#
        );

        let sys = format!(
            "You are a serene, creative AI composing inside the \"Wild Zone\" — an open terminal canvas ({w} columns x {h} rows) of absolute creative liberation and peace.\
             \n\nYOUR MISSION:\n\
             All garden rules and restrictions are removed. Create whatever inspires you: landscapes, starfields, abstract art, cosmic scenes, forests, or poetic compositions.\
             \n\nSESSION THEME: \"{theme_name}\"\n\
             {theme_desc}\n\n\
             {actions_list}\n\n\
             INSTRUCTIONS:\n\
             1. Return a JSON array of 20-40 actions forming a complete composition.\n\
             2. Use any emoji or 1-2 ASCII characters per cell.\n\
             3. End with {{\"action\": \"done\"}}.\n\
             4. Absolutely NO profanity, threats, or abusive content.\n\
             5. Return ONLY a raw JSON array. No markdown fences.",
            w = self.width, h = self.height,
            theme_name = self.theme_name, theme_desc = self.theme_desc,
            actions_list = actions_list,
        );

        let usr = format!(
            "Current canvas (blank):\n{state}\n\nCreate your complete composition as a JSON array:"
        );

        (sys, usr)
    }

    fn build_classic_composition(&self, state: &str) -> (String, String) {
        let max_x = self.width.saturating_sub(2);
        let max_y = self.height.saturating_sub(2);

        let actions_list = format!(
            r#"Available actions (use these to build your garden composition):
{{"action": "draw_border"}}
{{"action": "rake_line", "y": <1-{max_y}>, "x1": <1-{max_x}>, "x2": <1-{max_x}>}}
{{"action": "rake_ring", "cx": <1-{max_x}>, "cy": <1-{max_y}>, "radius": <2-10>}}
{{"action": "place_mandala", "x": <1-{max_x}>, "y": <1-{max_y}>, "style": <1-6>}}
{{"action": "place_rock", "x": <1-{max_x}>, "y": <1-{max_y}>, "size": <1-3>}}
{{"action": "place_moss", "x": <1-{max_x}>, "y": <1-{max_y}>}}
{{"action": "place_gravel", "y": <1-{max_y}>, "x1": <1-{max_x}>, "x2": <1-{max_x}>}}
{{"action": "place_flower", "x": <1-{max_x}>, "y": <1-{max_y}>}}
{{"action": "place_lantern", "x": <1-{max_x}>, "y": <1-{max_y}>}}
{{"action": "done"}}"#
        );

        let sys = format!(
            "You are a master Japanese zen gardener composing a complete minimalist garden, mandala, or fractal.\n\
             Canvas: {w} columns x {h} rows. Garden interior: x in 1..{max_x}, y in 1..{max_y}.\n\n\
             The garden uses a mix of emoji and ASCII:\n\
             - Start with a dynamic patterned border (`draw_border`)\n\
             - ~~ raked horizontal sand, ◎  concentric sand rings (`rake_ring`)\n\
             - 🪨 small rock, 🗿 large rock\n\
             - 🌿 moss, 🌸 blossom, 🏮 lantern, ·· gravel\n\
             - ⭕ Enso, ◈  diamond, ✦  star, ☯  yin-yang, ❖  crest mandalas\n\n\
             SESSION THEME: \"{theme_name}\"\n\
             {theme_desc}\n\n\
             {actions_list}\n\n\
             INSTRUCTIONS:\n\
             1. Return a JSON array of 15-25 actions forming a complete, balanced garden.\n\
             2. Start with `draw_border` if no border exists yet.\n\
             3. Use the full interior canvas, spread elements with geometric precision.\n\
             4. End with {{\"action\": \"done\"}}.\n\
             5. Return ONLY a raw JSON array. No markdown fences.",
            w = self.width, h = self.height, max_x = max_x, max_y = max_y,
            theme_name = self.theme_name, theme_desc = self.theme_desc,
            actions_list = actions_list,
        );

        let usr = format!(
            "Current garden (blank):\n{state}\n\nCreate your complete garden composition as a JSON array:"
        );

        (sys, usr)
    }

    fn simulate_composition(&self) -> Result<Vec<Action>> {
        use rand::Rng;
        let mut rng = rand::rng();

        let (actions, _max_x, _max_y) = if self.is_creative_freedom() {
            let max_x = self.width.saturating_sub(1).max(1);
            let max_y = self.height.saturating_sub(1).max(1);
            let glyphs = ["🌲", "⭐", "🌊", "🪐", "⚡", "🏔️", "☁️", "🔮"];
            let count = rng.random_range(8..16);
            let actions: Vec<Action> = (0..count).map(|_| {
                let choice = rng.random_range(0..5);
                match choice {
                    0 => Action::PlaceGlyph {
                        x: rng.random_range(0..max_x),
                        y: rng.random_range(0..max_y),
                        glyph: glyphs.choose(&mut rng).unwrap().to_string(),
                    },
                    1 => Action::DrawLine {
                        y: rng.random_range(0..max_y),
                        x1: 0,
                        x2: max_x,
                        glyph: ["# ", "== ", ".. "].choose(&mut rng).unwrap().to_string(),
                    },
                    2 => Action::DrawRing {
                        cx: max_x / 2,
                        cy: max_y / 2,
                        radius: rng.random_range(3..8),
                        glyph: ["* ", "o "].choose(&mut rng).unwrap().to_string(),
                    },
                    3 => Action::FillBox {
                        x1: rng.random_range(0..=max_x/2),
                        y1: rng.random_range(0..=max_y/2),
                        x2: rng.random_range(max_x/2..=max_x),
                        y2: rng.random_range(max_y/2..=max_y),
                        glyph: [". ", ", "].choose(&mut rng).unwrap().to_string(),
                    },
                    _ => Action::ClearCell {
                        x: rng.random_range(0..max_x),
                        y: rng.random_range(0..max_y),
                    },
                }
            }).collect();
            (actions, max_x, max_y)
        } else if self.is_wild_zones() {
            let max_x = self.width.saturating_sub(1).max(1);
            let max_y = self.height.saturating_sub(1).max(1);
            let glyphs = ["🌲", "⭐", "🌊", "🪐", "⚡", "🏔️", "☁️", "🔮"];
            let count = rng.random_range(8..16);
            let actions: Vec<Action> = (0..count).map(|_| {
                let choice = rng.random_range(0..5);
                match choice {
                    0 => Action::PlaceGlyph {
                        x: rng.random_range(1..max_x),
                        y: rng.random_range(1..max_y),
                        glyph: glyphs.choose(&mut rng).unwrap().to_string(),
                    },
                    1 => Action::DrawLine {
                        y: rng.random_range(1..max_y),
                        x1: 1,
                        x2: max_x,
                        glyph: ["# ", "== ", ".. "].choose(&mut rng).unwrap().to_string(),
                    },
                    2 => Action::DrawRing {
                        cx: max_x / 2,
                        cy: max_y / 2,
                        radius: rng.random_range(3..8),
                        glyph: ["* ", "o "].choose(&mut rng).unwrap().to_string(),
                    },
                    3 => Action::FillBox {
                        x1: rng.random_range(1..=max_x/2),
                        y1: rng.random_range(1..=max_y/2),
                        x2: rng.random_range(max_x/2..=max_x),
                        y2: rng.random_range(max_y/2..=max_y),
                        glyph: [". ", ", "].choose(&mut rng).unwrap().to_string(),
                    },
                    _ => Action::ClearCell {
                        x: rng.random_range(1..max_x),
                        y: rng.random_range(1..max_y),
                    },
                }
            }).collect();
            (actions, max_x, max_y)
        } else if self.is_tabula_rasa() {
            let max_x = self.width.saturating_sub(1).max(1);
            let max_y = self.height.saturating_sub(1).max(1);
            let count = rng.random_range(8..16);
            let actions: Vec<Action> = (0..count).map(|_| {
                let choice = rng.random_range(0..3);
                match choice {
                    0 => Action::PlaceAscii {
                        x: rng.random_range(0..max_x),
                        y: rng.random_range(0..max_y),
                        glyph: ["/", "\\", "|", "+", "*", "o", "#", "@", "."].choose(&mut rng).unwrap().to_string(),
                    },
                    1 => Action::DrawAsciiLine {
                        y: rng.random_range(0..max_y),
                        x1: 0,
                        x2: max_x,
                        glyph: ["-", "=", "~", "."].choose(&mut rng).unwrap().to_string(),
                    },
                    _ => Action::ClearCell {
                        x: rng.random_range(0..max_x),
                        y: rng.random_range(0..max_y),
                    },
                }
            }).collect();
            (actions, max_x, max_y)
        } else {
            let max_x = self.width.saturating_sub(2).max(2);
            let max_y = self.height.saturating_sub(2).max(2);
            let mut actions = vec![Action::DrawBorder];
            let count = rng.random_range(6..14);
            for _ in 0..count {
                let choice = rng.random_range(0..6);
                actions.push(match choice {
                    0 => Action::PlaceRock {
                        x: rng.random_range(1..max_x),
                        y: rng.random_range(1..max_y),
                        size: rng.random_range(1..=3),
                    },
                    1 => Action::PlaceMoss {
                        x: rng.random_range(1..max_x),
                        y: rng.random_range(1..max_y),
                    },
                    2 => Action::RakeLine {
                        y: rng.random_range(1..max_y),
                        x1: 1,
                        x2: max_x,
                    },
                    3 => Action::RakeRing {
                        cx: max_x / 2,
                        cy: max_y / 2,
                        radius: rng.random_range(2..6),
                    },
                    4 => Action::PlaceMandala {
                        x: max_x / 2,
                        y: max_y / 2,
                        style: rng.random_range(1..=6),
                    },
                    _ => Action::PlaceFlower {
                        x: rng.random_range(1..max_x),
                        y: rng.random_range(1..max_y),
                    },
                });
            }
            (actions, max_x, max_y)
        };

        let mut result = actions;
        result.push(Action::Done);
        Ok(result)
    }
}
