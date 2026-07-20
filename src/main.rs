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
    let theme = gardener.theme_name().to_string();

    println!("\x1b[2J\x1b[H");
    println!("🌿 karesansui — Theme: \"{theme}\"\n");
    println!("   the LLM is tending the garden...\n");

    let mut consecutive_errors = 0;
    let mut border_drawn = false;
    let mut action_num: usize = 0;

    for _ in 0..MAX_ACTIONS {
        let state = garden.render();
        print!("\x1b[2J\x1b[H");
        print!("🌿 karesansui — Theme: \"{theme}\"  [action {action_num}]\n\n{state}");
        std::io::Write::flush(&mut std::io::stdout())?;

        let action = match gardener.next_action(&state, border_drawn, action_num).await {
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
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        // Skip duplicate draw_border calls.
        if matches!(action, Action::DrawBorder) && border_drawn {
            continue;
        }

        match action {
            Action::PlaceRock { x, y, size } => garden.place_rock(x, y, size),
            Action::RakeLine { y, x1, x2 } => garden.rake_line(y, x1, x2),
            Action::PlaceMoss { x, y } => garden.place_moss(x, y),
            Action::PlaceGravel { y, x1, x2 } => garden.place_gravel(y, x1, x2),
            Action::PlaceFlower { x, y } => garden.place_flower(x, y),
            Action::PlaceLantern { x, y } => garden.place_lantern(x, y),
            Action::DrawBorder => {
                garden.draw_border();
                border_drawn = true;
            }
            Action::Done => {
                let final_state = garden.render();
                print!("\x1b[2J\x1b[H");
                print!("🌿 karesansui — \"{theme}\" — complete\n\n{final_state}");
                std::io::Write::flush(&mut std::io::stdout())?;
                println!("The garden is complete. 🍃");
                return Ok(());
            }
        }

        action_num += 1;
        tokio::time::sleep(TICK).await;
    }

    // Final render if we hit the cap.
    let final_state = garden.render();
    print!("\x1b[2J\x1b[H");
    println!("🌿 karesansui — \"{theme}\" — session limit reached\n");
    print!("{final_state}");
    Ok(())
}
