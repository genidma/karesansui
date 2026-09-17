mod garden;
mod llm;
mod openrouter;
mod vec;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use garden::{Action, Garden};
use llm::Composer;
use tokio::signal;

#[derive(Parser, Debug, Clone)]
#[command(name = "karesansui")]
#[command(about = "A creative terminal ASCII art generator, powered by an LLM.")]
pub struct CliArgs {
    /// Grid width in terminal columns (default: 48)
    #[arg(short, long, default_value_t = 48)]
    pub width: usize,

    /// Grid height in terminal rows (default: 20)
    #[arg(long, default_value_t = 20)]
    pub height: usize,

    /// Milliseconds between each action animation step (default: 80)
    #[arg(short, long, default_value_t = 80)]
    pub pace: u64,

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

    let args = CliArgs::parse();

    let _clean_exit = CleanExit;
    crossterm::execute!(std::io::stdout(), crossterm::cursor::Hide)?;

    let model = std::env::var("LLM_MODEL")
        .or_else(|_| std::env::var("OPENROUTER_MODEL"))
        .unwrap_or_else(|_| "tencent/hy3:free".to_string());

    let width = args.width;
    let height = args.height;

    let shutdown = Arc::new(AtomicBool::new(false));
    let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);
    {
        let shutdown_signal = shutdown.clone();
        let cancel_tx = cancel_tx.clone();
        tokio::spawn(async move {
            loop {
                signal::ctrl_c().await.ok();
                shutdown_signal.store(true, Ordering::SeqCst);
                let _ = cancel_tx.send(true);
            }
        });
    }
    let mut interrupted = false;

    let mut garden = Garden::new(width, height);
    let composer = Composer::new(&model, width, height, args.dry_run)?;
    let is_nvidia = composer.is_nvidia();

    const MAX_PIECES: u32 = 10;

    for _ in 0..MAX_PIECES {
        if shutdown.load(Ordering::SeqCst) { break; }

        crossterm::execute!(std::io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All), crossterm::cursor::MoveTo(0, 0))?;
        let theme_label = "🎨 Creative Freedom";
        println!("{theme_label}\n");
        println!("   ✨ The LLM is composing a complete ASCII art piece...\n");

        // Wait for the LLM to compose the full artwork — keep trying until success
        let mut compose_retries = 0u32;
        const MAX_COMPOSE_RETRIES: u32 = 5;
        let mut actions = Vec::new();
        loop {
            if shutdown.load(Ordering::SeqCst) { break; }
            if compose_retries >= MAX_COMPOSE_RETRIES {
                log::error!("Max compose retries ({MAX_COMPOSE_RETRIES}) exceeded. Skipping to next piece.");
                break;
            }
            if !is_nvidia {
                println!("⏳ Asking the LLM to compose a complete piece...");
            }
            let start = Instant::now();
            // Print elapsed every 30s during the LLM call so user knows it's still working
            let heartbeat = {
                let shutdown = shutdown.clone();
                tokio::spawn(async move {
                    for i in 1..20 {
                        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                        if shutdown.load(Ordering::SeqCst) { break; }
                        log::info!("Still waiting for LLM response... (elapsed: {}s)", i * 30);
                    }
                })
            };
            let compose = composer.compose_artwork();
            tokio::pin!(compose);
            tokio::select! {
                result = &mut compose => {
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
                }
                _ = cancel_rx.changed() => {
                    heartbeat.abort();
                    log::info!("Ctrl+C received — aborting LLM composition.");
                    interrupted = true;
                    break;
                }
            }
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(5 * compose_retries as u64)) => {}
                _ = cancel_rx.changed() => {
                    log::info!("Ctrl+C received — skipping retry backoff.");
                    interrupted = true;
                    break;
                }
            }
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
                "{theme_label} — [action {}/{} — ✨ creating...]",
                i + 1, action_count
            );

            if args.step {
                garden.render_screen(&header, args.no_color)?;
                log::info!("Action {}/{action_count}: {action:?}. Press Enter for next action...", i + 1);
                let mut line = String::new();
                std::io::stdin().read_line(&mut line).ok();
            }

            let exec = garden.execute_action(action, &header, args.no_color);
            tokio::pin!(exec);
            tokio::select! {
                r = &mut exec => { r?; }
                _ = cancel_rx.changed() => {
                    log::info!("Ctrl+C received — stopping action execution.");
                    interrupted = true;
                    break;
                }
            }
        }

        if let Some(ref snapshot_path) = args.snapshot {
            let _ = std::fs::write(snapshot_path, garden.render_colored(args.no_color));
        }

        // Admire the final piece
        let admire_secs = if args.admire == 0 { u64::MAX } else { args.admire };
        for remaining in (1..=admire_secs).rev() {
            if shutdown.load(Ordering::SeqCst) { break; }
            let suffix = if args.admire == 0 { "∞ until Ctrl+C" } else { &format!("{remaining}s until next piece") };
            let h = format!("{theme_label} — Complete! 💤 admiring ({suffix})",);
            garden.render_screen(&h, args.no_color)?;
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {}
                _ = cancel_rx.changed() => {
                    interrupted = true;
                    break;
                }
            }
        }

        // Reset canvas for next piece
        garden.reset();
    }

    crossterm::execute!(std::io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All), crossterm::cursor::MoveTo(0, 0))?;
    let farewell = if interrupted {
        "🌿 karesansui — Interrupted. See you next time!"
    } else {
        "🌿 karesansui — Done. See you next time!"
    };
    println!("{farewell}");
    Ok(())
}