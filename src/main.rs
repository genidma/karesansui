mod canvas;
mod color;
mod garden;
mod gridwright_runner;
mod llm;
mod openrouter;
mod pixel_art;
mod vec;

use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use garden::{Action, Garden};
use gridwright_runner::GridwrightRunner;
use llm::{Gardener, THEMES};
use pixel_art::GridwrightConfig;
use tokio::signal;

#[derive(Parser, Debug, Clone)]
#[command(name = "karesansui")]
#[command(about = "A creative terminal ASCII art generator, tended by a turtle and an LLM.")]
pub struct CliArgs {
    /// Choose a specific theme by name or index (1-21), or "random", or "classic"
    #[arg(short, long)]
    pub theme: Option<String>,

    /// Grid width in terminal columns (default: 48)
    #[arg(short, long, default_value_t = 48)]
    pub width: usize,

    /// Grid height in terminal rows (default: 20)
    #[arg(long, default_value_t = 20)]
    pub height: usize,

    /// Milliseconds between each action animation step (default: 80)
    #[arg(short, long, default_value_t = 80)]
    pub pace: u64,

    /// Interactive menu mode to select themes and settings on startup
    #[arg(short, long, default_value_t = false)]
    pub interactive: bool,

    /// Offline simulation without making LLM API calls
    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,

    /// Single-step mode: press Enter between each action
    #[arg(short, long, default_value_t = false)]
    pub step: bool,

    /// Dump final garden state to file on completion
    #[arg(long)]
    pub snapshot: Option<String>,

    /// Disable faint crossterm coloring and use plain text output
    #[arg(long, default_value_t = false)]
    pub no_color: bool,

    /// Seconds to admire the completed artwork before starting the next piece (default: 20, 0 to admire forever until Ctrl+C)
    #[arg(long, default_value_t = 20)]
    pub admire: u64,
}

/// Present a clean interactive menu for picking theme and settings.
fn interactive_menu(args: &mut CliArgs) -> Result<()> {
    println!("\x1b[2J\x1b[H");
    println!("🎋----------------------------------------------------------------------🎋");
    println!("   karesansui (枯山水) — ASCII Art & Zen Garden CLI");
    println!("🎋----------------------------------------------------------------------🎋\n");
    println!("Choose your garden theme:");

    let mid = (THEMES.len() + 1) / 2;
    for i in 0..mid {
        let left_num = i + 1;
        let left_name = THEMES[i].0;
        let left_str = format!("[{left_num}] {left_name}");

        if i + mid < THEMES.len() {
            let right_num = i + mid + 1;
            let right_name = THEMES[i + mid].0;
            let right_str = format!("[{right_num}] {right_name}");
            println!("  {left_str:<38} {right_str}");
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
    print!("Enter animation speed in ms (lower = faster) [default: {}]: ", args.pace);
    std::io::Write::flush(&mut std::io::stdout())?;
    if reader.read_line(&mut line).is_ok() && !line.trim().is_empty() {
        if let Ok(p) = line.trim().parse::<u64>() {
            args.pace = p;
        }
    }

    line.clear();
    print!("Enable step mode? (y/N) — pause between each action to steer the LLM: ");
    std::io::Write::flush(&mut std::io::stdout())?;
    if reader.read_line(&mut line).is_ok() {
        args.step = line.trim().eq_ignore_ascii_case("y") || line.trim().eq_ignore_ascii_case("yes");
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

    let model = std::env::var("LLM_MODEL")
        .or_else(|_| std::env::var("OPENROUTER_MODEL"))
        .unwrap_or_else(|_| "tencent/hy3:free".to_string());

    let gridwright_api_key = if args.dry_run {
        String::new()
    } else {
        std::env::var("LLM_API_KEY")
            .or_else(|_| std::env::var("OPENROUTER_API_KEY"))
            .map_err(|_| anyhow::anyhow!("LLM_API_KEY or OPENROUTER_API_KEY not set (add it to .env)"))?
    };

    let width = args.width;
    let height = args.height;

    let anim_delay = Duration::from_millis(args.pace);

    let shutdown = Arc::new(AtomicBool::new(false));
    {
        let shutdown_signal = shutdown.clone();
        tokio::spawn(async move {
            signal::ctrl_c().await.ok();
            shutdown_signal.store(true, Ordering::SeqCst);
        });
    }

    let mut garden = Garden::new(width, height);
    let gardener = Gardener::new(&model, width, height, args.theme.as_deref(), args.dry_run)?;

    let theme = gardener.theme_name().to_string();
    let is_creative = gardener.is_creative_freedom();
    let is_tabula = gardener.is_tabula_rasa();
    let is_wild = gardener.is_wild_zones();

    const MAX_PIECES: u32 = 10;
    let _piece_num = 0u32;

    if theme.contains("Gridwright") {
        let gw_w = if width != 48 || height != 20 { width } else { 16 };
        let gw_h = if width != 48 || height != 20 { height } else { 16 };
        let config = GridwrightConfig::new(gw_w, gw_h)
            .with_subject("A bold, balanced pixel portrait with chunky blocks and strong negative space")
            .with_palette("gridwright_spec")
            .with_composition("Balanced, chunky blocks, hard edges, visible pixels, strong negative space")
            .with_max_actions(20);
        let mut runner = GridwrightRunner::new(gridwright_api_key, model, config, args.dry_run)
            .with_pace(Duration::from_secs(1))
            .with_step(args.step)
            .with_no_color(args.no_color)
            .with_snapshot_path(args.snapshot.clone());
        let canvas = runner.run().await?;
        let rendered = if args.no_color {
            canvas.render()
        } else {
            canvas.render_with_colors()
        };
        if let Some(snapshot_path) = args.snapshot.as_deref() {
            std::fs::write(snapshot_path, &rendered)?;
        }
        println!("🎨 Gridwright — runtime LLM pixel art\n");
        println!("{rendered}");
        return Ok(());
    }
    for _ in 0..MAX_PIECES {
        if shutdown.load(Ordering::SeqCst) { break; }

        // Display theme intro
        crossterm::execute!(std::io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All), crossterm::cursor::MoveTo(0, 0))?;
        let theme_label = if is_creative { "🎨 Creative Freedom" } else if is_tabula { "✨ Tabula Rasa" } else if is_wild { "🌊 Wild Zones" } else { "🌿 karesansui" };
        println!("{theme_label} — Theme: \"{theme}\"\n");
        let desc = if is_creative { "The turtle is composing a complete ASCII art masterpiece..." } else if is_tabula { "[*] The ASCII muse is composing a pure ASCII artwork..." } else if is_wild { "The turtle is composing a wild creation..." } else { "The turtle is composing a zen garden..." };
        println!("   🐢 {desc}\n");

        // Wait for the LLM to compose the full artwork — keep trying until success
        let mut compose_retries = 0u32;
        const MAX_COMPOSE_RETRIES: u32 = 5;
        let mut actions = Vec::new();
        let is_nvidia = gardener.is_nvidia();
        loop {
            if shutdown.load(Ordering::SeqCst) { break; }
            if compose_retries >= MAX_COMPOSE_RETRIES {
                log::error!("Max compose retries ({MAX_COMPOSE_RETRIES}) exceeded. Skipping to next piece.");
                break;
            }
            if !is_nvidia {
                println!("⏳ Asking the LLM to compose a complete piece...");
            }
            let state = garden.render();
            let start = Instant::now();
            // Print elapsed every 30s during the LLM call so user knows it's still working
            let heartbeat = {
                let shutdown = shutdown.clone();
                tokio::spawn(async move {
                    for i in 1..20 {
                        tokio::time::sleep(Duration::from_secs(30)).await;
                        if shutdown.load(Ordering::SeqCst) { break; }
                        log::info!("Still waiting for LLM response... (elapsed: {}s)", i * 30);
                    }
                })
            };
            let result = gardener.compose_artwork(&state).await;
            heartbeat.abort();
            match result {
                Ok(a) if !a.is_empty() => {
                    log::info!("LLM composed {} actions in {:.1}s", a.len(), start.elapsed().as_secs_f64());
                    actions = a;
                    break;
                }
                Ok(_) => {
                    compose_retries += 1;
                    log::warn!("LLM returned empty action list. Retrying... (attempt {compose_retries}/{MAX_COMPOSE_RETRIES})");
                }
                Err(e) => {
                    compose_retries += 1;
                    log::warn!("LLM composition failed: {e}. Retrying... (attempt {compose_retries}/{MAX_COMPOSE_RETRIES})");
                }
            }
            tokio::time::sleep(Duration::from_secs(5 * compose_retries as u64)).await;
        };
        
        if actions.is_empty() {
            garden.reset();
            continue;
        }

        // Execute all actions in sequence
        let action_count = actions.len();
        for (i, action) in actions.iter().enumerate() {
            if shutdown.load(Ordering::SeqCst) { break; }
            if matches!(action, Action::Done) { break; }

            let header = format!(
                "{theme_label} — \"{theme}\"  [action {}/{} — 🐢 creating...]",
                i + 1, action_count
            );

            if args.step {
                garden.render_screen(&header, args.no_color)?;
                log::info!("Action {}/{action_count}: {action:?}. Press Enter for next action...", i + 1);
                let mut line = String::new();
                std::io::stdin().read_line(&mut line).ok();
            }

            garden.execute_action(&action, &header, args.no_color).await?;
            tokio::time::sleep(anim_delay).await;
        }

        if let Some(ref snapshot_path) = args.snapshot {
            let _ = std::fs::write(snapshot_path, garden.render_colored(args.no_color));
        }

        // Admire the final piece
        garden.turtle_glyph = "💤";
        let admire_secs = if args.admire == 0 { u64::MAX } else { args.admire };
        for remaining in (1..=admire_secs).rev() {
            if shutdown.load(Ordering::SeqCst) { break; }
            let suffix = if args.admire == 0 { "∞ until Ctrl+C" } else { &format!("{remaining}s until next piece") };
            let h = format!("{theme_label} — \"{theme}\" — Complete! 💤 admiring ({suffix})",);
            garden.render_screen(&h, args.no_color)?;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        // Reset canvas for next piece
        garden.reset();
    }

    crossterm::execute!(std::io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All), crossterm::cursor::MoveTo(0, 0))?;
    println!("🌿 karesansui — Interrupted. See you next time!");
    Ok(())
}
