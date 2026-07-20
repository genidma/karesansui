mod garden;
mod llm;

use std::io::BufRead;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use garden::{Action, Garden, GRAVEL, RAKED};
use llm::{Gardener, THEMES};

/// Session duration: 30 minutes per garden before automatic reset.
const SESSION_DURATION: Duration = Duration::from_secs(30 * 60);

#[derive(Parser, Debug, Clone)]
#[command(name = "karesansui")]
#[command(about = "A minimalist ASCII & emoji zen garden, mandala & fractal generator tended by a turtle and an LLM.")]
pub struct CliArgs {
    /// Choose a specific theme by name or index (1-19), or "random"
    #[arg(short, long)]
    pub theme: Option<String>,

    /// Grid width in terminal columns (default: 48)
    #[arg(short, long, default_value_t = 48)]
    pub width: usize,

    /// Grid height in terminal rows (default: 20)
    #[arg(long, default_value_t = 20)]
    pub height: usize,

    /// Seconds between normal LLM prompts (default: 6)
    #[arg(short, long, default_value_t = 6)]
    pub pace: u64,

    /// Seconds to rest after every 10 prompts (default: 30)
    #[arg(long, default_value_t = 30)]
    pub rest: u64,

    /// Interactive menu mode to select themes and settings on startup
    #[arg(short, long, default_value_t = false)]
    pub interactive: bool,
}

/// Helper: animate the turtle walking across the garden to (dest_x, dest_y).
async fn animate_walk(
    garden: &mut Garden,
    dest_x: usize,
    dest_y: usize,
    header: &str,
) -> Result<()> {
    let (mut tx, mut ty) = garden.turtle_pos.unwrap_or((1, 1));
    while tx != dest_x || ty != dest_y {
        if tx < dest_x {
            tx += 1;
        } else if tx > dest_x {
            tx -= 1;
        }
        if ty < dest_y {
            ty += 1;
        } else if ty > dest_y {
            ty -= 1;
        }
        garden.turtle_pos = Some((tx, ty));
        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
        std::io::Write::flush(&mut std::io::stdout())?;
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    Ok(())
}

/// Present a clean interactive menu for picking theme and settings.
fn interactive_menu(args: &mut CliArgs) -> Result<()> {
    println!("\x1b[2J\x1b[H");
    println!("🎋----------------------------------------------------------------------🎋");
    println!("   karesansui (枯山水) — Minimalist Zen Garden, Mandala & Fractal CLI");
    println!("🎋----------------------------------------------------------------------🎋\n");
    println!("Choose your garden theme:");
    
    let mid = (THEMES.len() + 1) / 2;
    for i in 0..mid {
        let left_num = i + 1;
        let left_name = THEMES[i].0;
        let left_tag = if left_num >= 13 { " (✨ NEW)" } else { "" };
        let left_str = format!("[{left_num}] {left_name}{left_tag}");

        if i + mid < THEMES.len() {
            let right_num = i + mid + 1;
            let right_name = THEMES[i + mid].0;
            let right_tag = if right_num >= 13 { " (✨ NEW)" } else { "" };
            let right_str = format!("[{right_num}] {right_name}{right_tag}");
            println!("  {left_str:<36} {right_str}");
        } else {
            println!("  {left_str}");
        }
    }
    println!("  [0]  🎲 Random Theme\n");

    let stdin = std::io::stdin();
    let mut reader = stdin.lock();

    print!("Enter theme number (0-{}) [default: 0]: ", THEMES.len());
    std::io::Write::flush(&mut std::io::stdout())?;
    let mut line = String::new();
    if reader.read_line(&mut line).is_ok() && !line.trim().is_empty() {
        let trimmed = line.trim();
        if trimmed != "0" && !trimmed.eq_ignore_ascii_case("random") {
            args.theme = Some(trimmed.to_string());
        }
    }

    line.clear();
    print!("Enter pace (seconds between moves) [default: {}]: ", args.pace);
    std::io::Write::flush(&mut std::io::stdout())?;
    if reader.read_line(&mut line).is_ok() && !line.trim().is_empty() {
        if let Ok(p) = line.trim().parse::<u64>() {
            args.pace = p;
        }
    }

    line.clear();
    print!("Enter rest (seconds after 10 moves) [default: {}]: ", args.rest);
    std::io::Write::flush(&mut std::io::stdout())?;
    if reader.read_line(&mut line).is_ok() && !line.trim().is_empty() {
        if let Ok(r) = line.trim().parse::<u64>() {
            args.rest = r;
        }
    }

    println!("\n✨ Settings saved! The turtle (`🐢`) is getting ready...\n");
    std::thread::sleep(Duration::from_secs(1));
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let mut args = CliArgs::parse();

    if args.interactive {
        interactive_menu(&mut args)?;
    }

    let model = std::env::var("OPENROUTER_MODEL")
        .unwrap_or_else(|_| "tencent/hy3:free".to_string());

    let width = args.width;
    let height = args.height;
    let pace = args.pace;
    let rest = args.rest;

    loop {
        let session_start = Instant::now();
        let mut garden = Garden::new(width, height);
        let gardener = Gardener::new(&model, width, height, args.theme.as_deref())?;
        let theme = gardener.theme_name().to_string();
        let border_name = garden.border_pattern.name;
        let is_tabula = gardener.is_tabula_rasa();
        let is_wild = gardener.is_wild_zones();

        let mut consecutive_errors = 0;
        let mut border_drawn = is_tabula || is_wild;
        let mut prompt_count: usize = 0;

        if is_tabula {
            garden.turtle_glyph = "[*]";
        } else if is_wild {
            garden.turtle_glyph = "🕊️";
        }

        println!("\x1b[2J\x1b[H");
        if is_tabula {
            println!("✨ Tabula Rasa — Theme: \"{theme}\"\n");
            println!("   [*] The ASCII muse is waking up to sketch across the canvas...\n");
        } else if is_wild {
            println!("🌊 Wild Zones — Theme: \"{theme}\"\n");
            println!("   🕊️ The dove of peace enters the unbound zone of absolute freedom and serenity...\n");
        } else {
            println!("🌿 karesansui — Theme: \"{theme}\" | Border: \"{border_name}\"\n");
            println!("   🐢 The turtle is waking up to tend the garden...\n");
        }
        tokio::time::sleep(Duration::from_secs(2)).await;

        while session_start.elapsed() < SESSION_DURATION {
            let state = garden.render();
            let header = if is_tabula {
                format!("✨ Tabula Rasa — Theme: \"{theme}\"  [prompt #{prompt_count}]")
            } else if is_wild {
                format!("🌊 Wild Zones — Theme: \"{theme}\"  [prompt #{prompt_count}]")
            } else {
                format!("🌿 karesansui — Theme: \"{theme}\" | Border: \"{border_name}\"  [prompt #{prompt_count}]")
            };
            print!("\x1b[2J\x1b[H{header}\n\n{state}");
            std::io::Write::flush(&mut std::io::stdout())?;

            let action = match gardener.next_action(&state, border_drawn, prompt_count).await {
                Ok(a) => {
                    consecutive_errors = 0;
                    a
                }
                Err(e) => {
                    consecutive_errors += 1;
                    if consecutive_errors >= 3 {
                        eprintln!("\ngardener error: {e}");
                        break;
                    }
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }
            };

            // Skip duplicate draw_border calls for standard themes.
            if matches!(action, Action::DrawBorder) && border_drawn && !is_wild {
                continue;
            }

            prompt_count += 1;
            let header = if is_tabula {
                format!("✨ Tabula Rasa — Theme: \"{theme}\"  [prompt #{prompt_count} — [*] sketching...]")
            } else if is_wild {
                format!("🌊 Wild Zones — Theme: \"{theme}\"  [prompt #{prompt_count} — 🕊️ creating...]")
            } else {
                format!("🌿 karesansui — Theme: \"{theme}\" | Border: \"{border_name}\"  [prompt #{prompt_count} — 🐢 building...]")
            };

            match action {
                Action::DrawBorder => {
                    // Turtle walks around the perimeter laying the pattern-based border.
                    for x in 0..width {
                        garden.draw_border_at(x, 0);
                        garden.turtle_pos = Some((x, 0));
                        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }
                    for y in 0..height {
                        garden.draw_border_at(width - 1, y);
                        garden.turtle_pos = Some((width - 1, y));
                        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }
                    for x in (0..width).rev() {
                        garden.draw_border_at(x, height - 1);
                        garden.turtle_pos = Some((x, height - 1));
                        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }
                    for y in (0..height).rev() {
                        garden.draw_border_at(0, y);
                        garden.turtle_pos = Some((0, y));
                        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }
                    garden.turtle_pos = Some((1, 1));
                    border_drawn = true;
                }
                Action::PlaceRock { x, y, size } => {
                    animate_walk(&mut garden, x, y, &header).await?;
                    garden.place_rock(x, y, size);
                    print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                    std::io::Write::flush(&mut std::io::stdout())?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::PlaceMoss { x, y } => {
                    animate_walk(&mut garden, x, y, &header).await?;
                    garden.place_moss(x, y);
                    print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                    std::io::Write::flush(&mut std::io::stdout())?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::PlaceFlower { x, y } => {
                    animate_walk(&mut garden, x, y, &header).await?;
                    garden.place_flower(x, y);
                    print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                    std::io::Write::flush(&mut std::io::stdout())?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::PlaceLantern { x, y } => {
                    animate_walk(&mut garden, x, y, &header).await?;
                    garden.place_lantern(x, y);
                    print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                    std::io::Write::flush(&mut std::io::stdout())?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::PlaceMandala { x, y, style } => {
                    animate_walk(&mut garden, x, y, &header).await?;
                    garden.place_mandala(x, y, style);
                    print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                    std::io::Write::flush(&mut std::io::stdout())?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::PlaceAscii { x, y, glyph } => {
                    animate_walk(&mut garden, x, y, &header).await?;
                    garden.place_ascii(x, y, &glyph);
                    print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                    std::io::Write::flush(&mut std::io::stdout())?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::DrawAsciiLine { y, x1, x2, glyph } => {
                    animate_walk(&mut garden, x1, y, &header).await?;
                    let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
                    let step_range: Vec<usize> = if x1 <= x2 {
                        (a..=b.min(width.saturating_sub(1))).collect()
                    } else {
                        (a..=b.min(width.saturating_sub(1))).rev().collect()
                    };
                    for x in step_range {
                        garden.turtle_pos = Some((x, y));
                        garden.place_ascii(x, y, &glyph);
                        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_millis(120)).await;
                    }
                }
                Action::PlaceGlyph { x, y, glyph } => {
                    animate_walk(&mut garden, x, y, &header).await?;
                    garden.place_glyph(x, y, &glyph);
                    print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                    std::io::Write::flush(&mut std::io::stdout())?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::DrawLine { y, x1, x2, glyph } => {
                    animate_walk(&mut garden, x1, y, &header).await?;
                    let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
                    let step_range: Vec<usize> = if x1 <= x2 {
                        (a..=b.min(width.saturating_sub(1))).collect()
                    } else {
                        (a..=b.min(width.saturating_sub(1))).rev().collect()
                    };
                    for x in step_range {
                        garden.turtle_pos = Some((x, y));
                        garden.place_glyph(x, y, &glyph);
                        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_millis(120)).await;
                    }
                }
                Action::DrawRing { cx, cy, radius, glyph } => {
                    let pts = garden.ring_points(cx, cy, radius);
                    if let Some(&(fx, fy)) = pts.first() {
                        animate_walk(&mut garden, fx, fy, &header).await?;
                    }
                    for (x, y) in pts {
                        garden.turtle_pos = Some((x, y));
                        garden.place_glyph(x, y, &glyph);
                        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
                Action::FillBox { x1, y1, x2, y2, glyph } => {
                    let (min_x, max_x) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
                    let (min_y, max_y) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
                    animate_walk(&mut garden, min_x, min_y, &header).await?;
                    for y in min_y..=max_y.min(height.saturating_sub(1)) {
                        for x in min_x..=max_x.min(width.saturating_sub(1)) {
                            garden.turtle_pos = Some((x, y));
                            garden.place_glyph(x, y, &glyph);
                            print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                            std::io::Write::flush(&mut std::io::stdout())?;
                            tokio::time::sleep(Duration::from_millis(60)).await;
                        }
                    }
                }
                Action::ClearCell { x, y } => {
                    animate_walk(&mut garden, x, y, &header).await?;
                    garden.clear_cell(x, y);
                    print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                    std::io::Write::flush(&mut std::io::stdout())?;
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
                Action::RakeLine { y, x1, x2 } => {
                    animate_walk(&mut garden, x1, y, &header).await?;
                    let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
                    let step_range: Vec<usize> = if x1 <= x2 {
                        (a..=b.min(width.saturating_sub(1))).collect()
                    } else {
                        (a..=b.min(width.saturating_sub(1))).rev().collect()
                    };
                    for x in step_range {
                        garden.turtle_pos = Some((x, y));
                        if garden.is_empty(x, y) {
                            garden.grid[y][x] = RAKED.to_string();
                        }
                        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_millis(120)).await;
                    }
                }
                Action::RakeRing { cx, cy, radius } => {
                    let pts = garden.ring_points(cx, cy, radius);
                    if let Some(first) = pts.first() {
                        animate_walk(&mut garden, first.0, first.1, &header).await?;
                    }
                    for (x, y) in pts {
                        garden.turtle_pos = Some((x, y));
                        if garden.is_empty(x, y) {
                            garden.grid[y][x] = RAKED.to_string();
                        }
                        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
                Action::PlaceGravel { y, x1, x2 } => {
                    animate_walk(&mut garden, x1, y, &header).await?;
                    let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
                    let step_range: Vec<usize> = if x1 <= x2 {
                        (a..=b.min(width.saturating_sub(1))).collect()
                    } else {
                        (a..=b.min(width.saturating_sub(1))).rev().collect()
                    };
                    for x in step_range {
                        garden.turtle_pos = Some((x, y));
                        if garden.is_empty(x, y) {
                            garden.grid[y][x] = GRAVEL.to_string();
                        }
                        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_millis(120)).await;
                    }
                }
                Action::Done => {
                    garden.turtle_glyph = if is_tabula { "[z]" } else if is_wild { "✨" } else { "💤" };
                    for remaining in (1..=20).rev() {
                        let h = if is_tabula {
                            format!("✨ Tabula Rasa — \"{theme}\" — Complete! [z] admiring ({remaining}s until reset)")
                        } else if is_wild {
                            format!("🌊 Wild Zones — \"{theme}\" — Complete! ✨ admiring ({remaining}s until reset)")
                        } else {
                            format!("🌿 karesansui — \"{theme}\" | Border: \"{border_name}\" — Complete! 💤 admiring ({remaining}s until reset)")
                        };
                        print!("\x1b[2J\x1b[H{h}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    break;
                }
            }

            // Rate limiting & pacing between prompts:
            let wait_secs = if prompt_count % 10 == 0 { rest } else { pace };
            garden.turtle_glyph = if is_tabula { "[z]" } else if is_wild { "✨" } else { "💤" };
            for remaining in (1..=wait_secs).rev() {
                if session_start.elapsed() >= SESSION_DURATION {
                    break;
                }
                let status = if prompt_count % 10 == 0 {
                    format!("[prompt #{prompt_count} — {} resting {rest}s rate-limit pause: {remaining}s remaining]", if is_tabula { "[z]" } else if is_wild { "✨" } else { "💤" })
                } else {
                    format!("[prompt #{prompt_count} — {} resting: {remaining}s until next move]", if is_tabula { "[z]" } else if is_wild { "✨" } else { "💤" })
                };
                let h = if is_tabula {
                    format!("✨ Tabula Rasa — Theme: \"{theme}\"  {status}")
                } else if is_wild {
                    format!("🌊 Wild Zones — Theme: \"{theme}\"  {status}")
                } else {
                    format!("🌿 karesansui — Theme: \"{theme}\" | Border: \"{border_name}\"  {status}")
                };
                print!("\x1b[2J\x1b[H{h}\n\n{}", garden.render());
                std::io::Write::flush(&mut std::io::stdout())?;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            garden.turtle_glyph = if is_tabula { "[*]" } else if is_wild { "🕊️" } else { "🐢" };
        }

        println!("\x1b[2J\x1b[H");
        println!("🌿 karesansui — 30-minute garden cycle complete or finished.");
        println!("   🔄 Starting a brand new garden in 3 seconds...\n");
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
