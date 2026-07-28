use anyhow::Result;
use rand::seq::IndexedRandom;

use crate::garden::Action;
use crate::openrouter::LlmClient;

/// How many prior JSON actions to include in each LLM prompt.
pub const RECENT_ACTIONS_LIMIT: usize = 10;

fn format_action_history(recent_actions: &[Action]) -> String {
    if recent_actions.is_empty() {
        return String::new();
    }
    let lines: Vec<String> = recent_actions
        .iter()
        .enumerate()
        .filter_map(|(i, action)| {
            serde_json::to_string(action)
                .ok()
                .map(|json| format!("{}. {json}", i + 1))
        })
        .collect();
    format!(
        "Recent actions (do NOT repeat these exact actions):\n{}\n\n",
        lines.join("\n")
    )
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
        let using_openrouter = std::env::var("LLM_API_URL")
            .or_else(|_| std::env::var("OPENROUTER_URL"))
            .map(|u| u.contains("openrouter.ai"))
            .unwrap_or(true);
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
                    THEMES
                        .iter()
                        .find(|(name, _)| name.to_lowercase().contains(&choice.to_lowercase()))
                        .copied()
                        .unwrap_or_else(|| {
                            log::warn!("theme '{choice}' not found; choosing randomly");
                            *THEMES.choose(&mut rng).unwrap()
                        })
                }
            }
            _ => *THEMES.choose(&mut rng).unwrap(),
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
        recent_actions: &[Action],
    ) -> Result<Action> {
        let history = format_action_history(recent_actions);
        let (system, user) = self.build_prompt(state, border_drawn, action_num, &history);

        if self.dry_run {
            return self.simulate_action(border_drawn, action_num);
        }

        let client = self.client.as_ref().unwrap();
        let content = client.call_raw(&system, &user, 0.8, "karesansui").await?;

        let action: Action = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("failed to parse LLM action '{content}': {e}"))?;
        Ok(action)
    }

    fn build_prompt(&self, state: &str, border_drawn: bool, action_num: usize, history: &str) -> (String, String) {
        if self.is_tabula_rasa() {
            self.build_tabula_prompt(state, action_num, history)
        } else if self.is_wild_zones() {
            self.build_wild_prompt(state, action_num, history)
        } else {
            self.build_classic_prompt(state, border_drawn, action_num, history)
        }
    }

    fn build_tabula_prompt(&self, state: &str, action_num: usize, history: &str) -> (String, String) {
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
            w = self.width, h = self.height, max_x = max_x, max_y = max_y,
            theme_name = self.theme_name, theme_desc = self.theme_desc,
            actions_block = actions_block, completion_hint = completion_hint,
        );
        let usr = format!(
            "{history}Current canvas (action #{action_num}):\n{state}\nNext action?",
            action_num = action_num,
        );
        (sys, usr)
    }

    fn build_wild_prompt(&self, state: &str, action_num: usize, history: &str) -> (String, String) {
        let max_x = self.width.saturating_sub(1);
        let max_y = self.height.saturating_sub(1);
        let completion_hint = if action_num >= 25 {
            "\nYou have created many elements. Consider calling done soon if your wild composition feels complete."
        } else {
            ""
        };
        let actions_block = format!(
            r#"Available actions (return ONE as raw JSON, no markdown, no extra text):
{{"action": "place_glyph", "x": <0-{max_x}>, "y": <0-{max_y}>, "glyph": "<any single emoji like '🌲','⭐','🌊','🪐','⚡','🏔️','☁️' or 1-2 ASCII chars like '# ','/**','/\\','||','..','==','++'>"}}
{{"action": "draw_line", "y": <0-{max_y}>, "x1": <0-{max_x}>, "x2": <0-{max_x}>, "glyph": "<any single emoji or 1-2 ASCII chars>"}}
{{"action": "draw_ring", "cx": <0-{max_x}>, "cy": <0-{max_y}>, "radius": <2-12>, "glyph": "<any single emoji or 1-2 ASCII chars>"}}
{{"action": "fill_box", "x1": <0-{max_x}>, "y1": <0-{max_y}>, "x2": <0-{max_x}>, "y2": <0-{max_y}>, "glyph": "<any single emoji or 1-2 ASCII chars>"}}
{{"action": "clear_cell", "x": <0-{max_x}>, "y": <0-{max_y}>}}
{{"action": "done"}}"#
        );
        let sys = format!(
            "You are a serene, creative AI composing inside the \"Wild Zone\" — an open terminal canvas ({w} columns x {h} rows) of absolute creative liberation and peace.\n\n\
             YOUR MISSION:\n\
             All concepts and code restrictions from other themes (zen gardens, raked sand, mandalas, and rigid borders) are completely removed. You are truly free.\n\
             You have absolute freedom across the entire grid (x: 0..{max_x}, y: 0..{max_y}). Create whatever inspires you: serene natural landscapes, celestial starfields, abstract generative textures, cosmic phenomena, peaceful forests, or poetic compositions.\n\
             You can freely place ANY standard emoji (`🌲`, `⭐`, `🌊`, `🪐`, `⚡`, `🏔️`, `☁️`, `🔮`, `🌌`, `🌿`, `🌙`) or ANY 1-2 ASCII characters (`# `, `/**`, `/\\`, `||`, `..`, `==`, `++`) anywhere using universal drawing actions (`place_glyph`, `draw_line`, `draw_ring`, `fill_box`, `clear_cell`).\n\n\
             SESSION THEME: \"{theme_name}\"\n\
             {theme_desc}\n\n\
             {actions_block}\n\n\
             RULES:\n\
             1. ABSOLUTE LIBERATION: No thematic boundaries, no pre-packaged garden elements, and no structural restrictions. You decide every shape, texture, and symbol.\n\
             2. STRICT SAFETY & SERENITY: Absolutely NO profanity, NO abusive language, and NO threatening content. Guided strictly by common sense, peace, calm, and serenity.\n\
             3. GRID MECHANICS: Every terminal grid cell is 2 columns wide. If placing an emoji (`🌲`, `⭐`), pass exactly 1 emoji per cell. If placing ASCII (`# `, `**`), pass exactly 1 or 2 ASCII characters per cell so they fit cleanly without distorting alignment.\n\
             4. Take your time over 15-35 prompts to build up your wild composition, then call `done` when complete.\n\
             5. NEVER repeat the exact same action. Each turn must introduce something unique.\n\
             6. Return ONLY one raw JSON object. No markdown fences.{completion_hint}",
            w = self.width, h = self.height, max_x = max_x, max_y = max_y,
            theme_name = self.theme_name, theme_desc = self.theme_desc,
            actions_block = actions_block, completion_hint = completion_hint,
        );
        let usr = format!(
            "{history}Current wild zone (action #{action_num}):\n{state}\nNext action?",
            action_num = action_num,
        );
        (sys, usr)
    }

    fn build_classic_prompt(&self, state: &str, border_drawn: bool, action_num: usize, history: &str) -> (String, String) {
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
            w = self.width, h = self.height, max_x = max_x, max_y = max_y,
            theme_name = self.theme_name, theme_desc = self.theme_desc,
            actions_block = actions_block, completion_hint = completion_hint,
        );

        let usr = format!(
            "{history}Current garden (action #{action_num}):\n{state}\nNext action?",
            action_num = action_num,
        );
        (sys, usr)
    }

    fn simulate_action(&self, border_drawn: bool, _action_num: usize) -> Result<Action> {
        use rand::seq::IndexedRandom;
        use rand::Rng;
        let mut rng = rand::rng();
        let max_x = self.width.saturating_sub(2).max(2);
        let max_y = self.height.saturating_sub(2).max(2);

        if self.is_wild_zones() {
            let choice = rng.random_range(0..5);
            return Ok(match choice {
                0 => Action::PlaceGlyph {
                    x: rng.random_range(1..max_x),
                    y: rng.random_range(1..max_y),
                    glyph: ["🌲", "⭐", "🌊", "🪐", "⚡", "🏔️", "☁️", "🔮"].choose(&mut rng).unwrap().to_string(),
                },
                1 => Action::DrawLine {
                    y: rng.random_range(1..max_y),
                    x1: 1,
                    x2: max_x,
                    glyph: ["# ", "== ", ".. ", "++ "].choose(&mut rng).unwrap().to_string(),
                },
                2 => Action::DrawRing {
                    cx: max_x / 2,
                    cy: max_y / 2,
                    radius: rng.random_range(2..6),
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
            });
        }

        if self.is_tabula_rasa() {
            let choice = rng.random_range(0..4);
            return Ok(match choice {
                0 => Action::PlaceAscii {
                    x: rng.random_range(1..max_x),
                    y: rng.random_range(1..max_y),
                    glyph: ["/", "\\", "|", "+", "*", "o", "#", "@", "."].choose(&mut rng).unwrap().to_string(),
                },
                1 => Action::DrawAsciiLine {
                    y: rng.random_range(1..max_y),
                    x1: 1,
                    x2: max_x,
                    glyph: ["-", "=", "~", "."].choose(&mut rng).unwrap().to_string(),
                },
                2 => Action::ClearCell {
                    x: rng.random_range(1..max_x),
                    y: rng.random_range(1..max_y),
                },
                _ => Action::DrawAsciiLine {
                    y: rng.random_range(1..max_y),
                    x1: 1,
                    x2: max_x,
                    glyph: ["=", "-", "."].choose(&mut rng).unwrap().to_string(),
                },
            });
        }

        if !border_drawn {
            return Ok(Action::DrawBorder);
        }

        let choice = rng.random_range(0..6);
        Ok(match choice {
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
        })
    }
}
