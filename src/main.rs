mod canvas;
mod color;
mod garden;
mod gridwright_runner;
mod llm;
mod pixel_art;
mod vec;

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

    /// Resume garden from saved state file across sessions
    #[arg(short, long, default_value_t = false)]
    pub resume: bool,

    /// Path to JSON state file for saving or resuming state
    #[arg(long)]
    pub state_file: Option<String>,

    /// Offline simulation without making OpenRouter API calls
    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,

    /// Single-step debug mode: run one prompt and wait for Enter between actions
    #[arg(short, long, default_value_t = false)]
    pub step: bool,

    /// Dump garden state/text to file on completion or step
    #[arg(long)]
    pub snapshot: Option<String>,

    /// Disable faint crossterm coloring and use plain text output
    #[arg(long, default_value_t = false)]
    pub no_color: bool,
}

fn render_screen(header: &str, garden: &Garden, no_color: bool) -> Result<()> {
    use crossterm::{cursor, terminal};
    use std::io::Write;
    let mut stdout = std::io::stdout();
    crossterm::queue!(stdout, cursor::Hide, cursor::MoveTo(0, 0))?;
    let full_text = format!("{header}\n\n{}", garden.render_colored(no_color));
    for line in full_text.lines() {
        crossterm::queue!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        writeln!(stdout, "{line}")?;
    }
    crossterm::queue!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown))?;
    stdout.flush()?;
    Ok(())
}

/// Helper: animate the turtle walking across the garden to (dest_x, dest_y).
async fn animate_walk(
    garden: &mut Garden,
    dest_x: usize,
    dest_y: usize,
    header: &str,
    no_color: bool,
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
        render_screen(header, garden, no_color)?;
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

struct CleanExit;
impl Drop for CleanExit {
    fn drop(&mut self) {
        let _ = crossterm::execute!(std::io::stdout(), crossterm::cursor::Show);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("karesansui v0.8.0 initializing...");

    let mut args = CliArgs::parse();

    if args.interactive {
        interactive_menu(&mut args)?;
    }

    let _clean_exit = CleanExit;
    crossterm::execute!(std::io::stdout(), crossterm::cursor::Hide)?;

    let model = std::env::var("OPENROUTER_MODEL")
        .unwrap_or_else(|_| "tencent/hy3:free".to_string());

    let width = args.width;
    let height = args.height;

    let pace_duration = if let Ok(ms_str) = std::env::var("KARESANSUI_TICK_MS") {
        if let Ok(ms) = ms_str.parse::<u64>() {
            Duration::from_millis(ms)
        } else {
            Duration::from_secs(args.pace)
        }
    } else {
        Duration::from_secs(args.pace)
    };

    let rest_duration = if let Ok(rest_str) = std::env::var("KARESANSUI_REST_SECS") {
        if let Ok(r) = rest_str.parse::<u64>() {
            Duration::from_secs(r)
        } else {
            Duration::from_secs(args.rest)
        }
    } else {
        Duration::from_secs(args.rest)
    };

    loop {
        let session_start = Instant::now();
        let (mut garden, mut prompt_count, theme_name, is_resumed) = if args.resume {
            let state_path = args.state_file.as_deref().unwrap_or("karesansui_state.json");
            match Garden::load_from_file(state_path) {
                Ok((g, p, t)) => {
                    log::info!("Resumed garden state from {state_path} (theme: {t}, prompt #{p})");
                    (g, p, t, true)
                }
                Err(e) => {
                    log::warn!("Could not load resume state from {state_path}: {e}. Creating new garden.");
                    let g = Garden::new(width, height);
                    (g, 0, String::new(), false)
                }
            }
        } else {
            let g = Garden::new(width, height);
            (g, 0, String::new(), false)
        };

        let gardener = if is_resumed {
            Gardener::new(&model, garden.width, garden.height, Some(&theme_name), args.dry_run)?
        } else {
            Gardener::new(&model, width, height, args.theme.as_deref(), args.dry_run)?
        };

        let theme = gardener.theme_name().to_string();
        let border_name = garden.border_pattern.name;
        let is_tabula = gardener.is_tabula_rasa();
        let is_wild = gardener.is_wild_zones();

        let mut consecutive_errors = 0;
        let mut border_drawn = is_tabula || is_wild || is_resumed;

        if is_tabula {
            garden.turtle_glyph = "[*]";
        } else {
            garden.turtle_glyph = "🐢";
        }

        if !is_resumed {
            crossterm::execute!(std::io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All), crossterm::cursor::MoveTo(0, 0))?;
            if is_tabula {
                println!("✨ Tabula Rasa — Theme: \"{theme}\"\n");
                println!("   [*] The ASCII muse is waking up to sketch across the canvas...\n");
            } else if is_wild {
                println!("🌊 Wild Zones — Theme: \"{theme}\"\n");
                println!("   🐢 The turtle enters the unbound zone of absolute freedom and serenity...\n");
            } else {
                println!("🌿 karesansui — Theme: \"{theme}\" | Border: \"{border_name}\"\n");
                println!("   🐢 The turtle is waking up to tend the garden...\n");
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        while session_start.elapsed() < SESSION_DURATION {
            let state = garden.render();
            let header = if is_tabula {
                format!("✨ Tabula Rasa — Theme: \"{theme}\"  [prompt #{prompt_count}]")
            } else if is_wild {
                format!("🌊 Wild Zones — Theme: \"{theme}\"  [prompt #{prompt_count}]")
            } else {
                format!("🌿 karesansui — Theme: \"{theme}\" | Border: \"{border_name}\"  [prompt #{prompt_count}]")
            };
            render_screen(&header, &garden, args.no_color)?;

            let action = match gardener.next_action(&state, border_drawn, prompt_count).await {
                Ok(a) => {
                    consecutive_errors = 0;
                    a
                }
                Err(e) => {
                    consecutive_errors += 1;
                    if consecutive_errors >= 3 {
                        log::error!("Gardener failed consistently: {e}");
                        break;
                    }
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }
            };

            if matches!(action, Action::DrawBorder) && border_drawn && !is_wild {
                continue;
            }

            prompt_count += 1;
            let header = if is_tabula {
                format!("✨ Tabula Rasa — Theme: \"{theme}\"  [prompt #{prompt_count} — [*] sketching...]")
            } else if is_wild {
                format!("🌊 Wild Zones — Theme: \"{theme}\"  [prompt #{prompt_count} — 🐢 creating...]")
            } else {
                format!("🌿 karesansui — Theme: \"{theme}\" | Border: \"{border_name}\"  [prompt #{prompt_count} — 🐢 building...]")
            };

            match action {
                Action::DrawBorder => {
                    for x in 0..width {
                        garden.draw_border_at(x, 0);
                        garden.turtle_pos = Some((x, 0));
                        render_screen(&header, &garden, args.no_color)?;
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }
                    for y in 0..height {
                        garden.draw_border_at(width - 1, y);
                        garden.turtle_pos = Some((width - 1, y));
                        render_screen(&header, &garden, args.no_color)?;
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }
                    for x in (0..width).rev() {
                        garden.draw_border_at(x, height - 1);
                        garden.turtle_pos = Some((x, height - 1));
                        render_screen(&header, &garden, args.no_color)?;
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }
                    for y in (0..height).rev() {
                        garden.draw_border_at(0, y);
                        garden.turtle_pos = Some((0, y));
                        render_screen(&header, &garden, args.no_color)?;
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }
                    garden.turtle_pos = Some((1, 1));
                    border_drawn = true;
                }
                Action::PlaceRock { x, y, size } => {
                    animate_walk(&mut garden, x, y, &header, args.no_color).await?;
                    garden.place_rock(x, y, size);
                    render_screen(&header, &garden, args.no_color)?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::PlaceMoss { x, y } => {
                    animate_walk(&mut garden, x, y, &header, args.no_color).await?;
                    garden.place_moss(x, y);
                    render_screen(&header, &garden, args.no_color)?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::PlaceFlower { x, y } => {
                    animate_walk(&mut garden, x, y, &header, args.no_color).await?;
                    garden.place_flower(x, y);
                    render_screen(&header, &garden, args.no_color)?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::PlaceLantern { x, y } => {
                    animate_walk(&mut garden, x, y, &header, args.no_color).await?;
                    garden.place_lantern(x, y);
                    render_screen(&header, &garden, args.no_color)?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::PlaceMandala { x, y, style } => {
                    animate_walk(&mut garden, x, y, &header, args.no_color).await?;
                    garden.place_mandala(x, y, style);
                    render_screen(&header, &garden, args.no_color)?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::PlaceAscii { x, y, glyph } => {
                    animate_walk(&mut garden, x, y, &header, args.no_color).await?;
                    garden.place_ascii(x, y, &glyph);
                    render_screen(&header, &garden, args.no_color)?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::DrawAsciiLine { y, x1, x2, glyph } => {
                    animate_walk(&mut garden, x1, y, &header, args.no_color).await?;
                    let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
                    let step_range: Vec<usize> = if x1 <= x2 {
                        (a..=b.min(width.saturating_sub(1))).collect()
                    } else {
                        (a..=b.min(width.saturating_sub(1))).rev().collect()
                    };
                    for x in step_range {
                        garden.turtle_pos = Some((x, y));
                        garden.place_ascii(x, y, &glyph);
                        render_screen(&header, &garden, args.no_color)?;
                        tokio::time::sleep(Duration::from_millis(120)).await;
                    }
                }
                Action::PlaceGlyph { x, y, glyph } => {
                    animate_walk(&mut garden, x, y, &header, args.no_color).await?;
                    garden.place_glyph(x, y, &glyph);
                    render_screen(&header, &garden, args.no_color)?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Action::DrawLine { y, x1, x2, glyph } => {
                    animate_walk(&mut garden, x1, y, &header, args.no_color).await?;
                    let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
                    let step_range: Vec<usize> = if x1 <= x2 {
                        (a..=b.min(width.saturating_sub(1))).collect()
                    } else {
                        (a..=b.min(width.saturating_sub(1))).rev().collect()
                    };
                    for x in step_range {
                        garden.turtle_pos = Some((x, y));
                        garden.place_glyph(x, y, &glyph);
                        render_screen(&header, &garden, args.no_color)?;
                        tokio::time::sleep(Duration::from_millis(120)).await;
                    }
                }
                Action::DrawRing { cx, cy, radius, glyph } => {
                    let pts = garden.ring_points(cx, cy, radius);
                    if let Some(&(fx, fy)) = pts.first() {
                        animate_walk(&mut garden, fx, fy, &header, args.no_color).await?;
                    }
                    for (x, y) in pts {
                        garden.turtle_pos = Some((x, y));
                        garden.place_glyph(x, y, &glyph);
                        render_screen(&header, &garden, args.no_color)?;
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
                Action::FillBox { x1, y1, x2, y2, glyph } => {
                    let (min_x, max_x) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
                    let (min_y, max_y) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
                    animate_walk(&mut garden, min_x, min_y, &header, args.no_color).await?;
                    for y in min_y..=max_y.min(height.saturating_sub(1)) {
                        for x in min_x..=max_x.min(width.saturating_sub(1)) {
                            garden.turtle_pos = Some((x, y));
                            garden.place_glyph(x, y, &glyph);
                            render_screen(&header, &garden, args.no_color)?;
                            tokio::time::sleep(Duration::from_millis(60)).await;
                        }
                    }
                }
                Action::ClearCell { x, y } => {
                    animate_walk(&mut garden, x, y, &header, args.no_color).await?;
                    garden.clear_cell(x, y);
                    render_screen(&header, &garden, args.no_color)?;
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
                Action::RakeLine { y, x1, x2 } => {
                    animate_walk(&mut garden, x1, y, &header, args.no_color).await?;
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
                        render_screen(&header, &garden, args.no_color)?;
                        tokio::time::sleep(Duration::from_millis(120)).await;
                    }
                }
                Action::RakeRing { cx, cy, radius } => {
                    let pts = garden.ring_points(cx, cy, radius);
                    if let Some(first) = pts.first() {
                        animate_walk(&mut garden, first.0, first.1, &header, args.no_color).await?;
                    }
                    for (x, y) in pts {
                        garden.turtle_pos = Some((x, y));
                        if garden.is_empty(x, y) {
                            garden.grid[y][x] = RAKED.to_string();
                        }
                        render_screen(&header, &garden, args.no_color)?;
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
                Action::PlaceGravel { y, x1, x2 } => {
                    animate_walk(&mut garden, x1, y, &header, args.no_color).await?;
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
                        render_screen(&header, &garden, args.no_color)?;
                        tokio::time::sleep(Duration::from_millis(120)).await;
                    }
                }
                Action::Done => {
                    garden.turtle_glyph = if is_tabula { "[z]" } else { "💤" };
                    for remaining in (1..=20).rev() {
                        let h = if is_tabula {
                            format!("✨ Tabula Rasa — \"{theme}\" — Complete! [z] admiring ({remaining}s until reset)")
                        } else if is_wild {
                            format!("🌊 Wild Zones — \"{theme}\" — Complete! 💤 admiring ({remaining}s until reset)")
                        } else {
                            format!("🌿 karesansui — \"{theme}\" | Border: \"{border_name}\" — Complete! 💤 admiring ({remaining}s until reset)")
                        };
                        render_screen(&h, &garden, args.no_color)?;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    break;
                }
            }

            if args.resume || args.state_file.is_some() {
                let state_path = args.state_file.as_deref().unwrap_or("karesansui_state.json");
                if let Err(e) = garden.save_to_file(state_path, prompt_count, &theme) {
                    log::warn!("Failed to save state to {state_path}: {e}");
                }
            }
            if let Some(ref snapshot_path) = args.snapshot {
                let _ = std::fs::write(snapshot_path, garden.render_colored(args.no_color));
            }
            if args.step {
                log::info!("Step completed (prompt #{prompt_count}). Exiting step mode.");
                return Ok(());
            }

            let wait_dur = if prompt_count % 10 == 0 { rest_duration } else { pace_duration };
            garden.turtle_glyph = if is_tabula { "[z]" } else { "💤" };
            for remaining in (1..=wait_dur.as_secs()).rev() {
                if session_start.elapsed() >= SESSION_DURATION {
                    break;
                }
                let status = if prompt_count % 10 == 0 {
                    format!("[prompt #{prompt_count} — {} resting {}s rate-limit pause: {remaining}s remaining]", if is_tabula { "[z]" } else { "💤" }, rest_duration.as_secs())
                } else {
                    format!("[prompt #{prompt_count} — {} resting: {remaining}s until next move]", if is_tabula { "[z]" } else { "💤" })
                };
                let h = if is_tabula {
                    format!("✨ Tabula Rasa — Theme: \"{theme}\"  {status}")
                } else if is_wild {
                    format!("🌊 Wild Zones — Theme: \"{theme}\"  {status}")
                } else {
                    format!("🌿 karesansui — Theme: \"{theme}\" | Border: \"{border_name}\"  {status}")
                };
                render_screen(&h, &garden, args.no_color)?;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            garden.turtle_glyph = if is_tabula { "[*]" } else { "🐢" };
        }

        crossterm::execute!(std::io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All), crossterm::cursor::MoveTo(0, 0))?;
        println!("🌿 karesansui — 30-minute garden cycle complete or finished.");
        println!("   🔄 Starting a brand new garden in 3 seconds...\n");
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
