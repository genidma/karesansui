mod garden;
mod llm;

use std::time::Duration;

use anyhow::Result;
use garden::{Action, Garden};
use llm::Gardener;

const WIDTH: usize = 48;
const HEIGHT: usize = 20;
/// Pacing between gardener actions — the "slow video game" feel.
const TICK: Duration = Duration::from_millis(1500);
/// Hard cap so it can't loop forever if the LLM never says done.
const MAX_ACTIONS: usize = 40;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let model = std::env::var("OPENROUTER_MODEL")
        .unwrap_or_else(|_| "tencent/hy3:free".to_string());

    let mut garden = Garden::new(WIDTH, HEIGHT);
    let gardener = Gardener::new(model, WIDTH, HEIGHT)?;

    println!("\x1b[2J\x1b[H"); // clear screen
    println!("🌿 karesansui — the LLM is tending the garden...\n");

    for _ in 0..MAX_ACTIONS {
        let state = garden.render();
        // Redraw in place.
        print!("\x1b[2J\x1b[H");
        print!("🌿 karesansui\n\n{state}");
        std::io::Write::flush(&mut std::io::stdout())?;

        let action = match gardener.next_action(&state).await {
            Ok(a) => a,
            Err(e) => {
                eprintln!("gardener error: {e}");
                break;
            }
        };

        match action {
            Action::PlaceRock { x, y, size } => garden.place_rock(x, y, size),
            Action::RakeLine { y, x1, x2 } => garden.rake_line(y, x1, x2),
            Action::DrawBorder => garden.draw_border(),
            Action::Done => {
                print!("\x1b[2J\x1b[H");
                print!("🌿 karesansui — complete\n\n{state}");
                std::io::Write::flush(&mut std::io::stdout())?;
                println!("\nThe garden is complete. 🍃");
                return Ok(());
            }
        }

        tokio::time::sleep(TICK).await;
    }

    // Final render if we hit the cap.
    print!("\x1b[2J\x1b[H");
    println!("🌿 karesansui — session limit reached\n");
    print!("{}", garden.render());
    Ok(())
}
