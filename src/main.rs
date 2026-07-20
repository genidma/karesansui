mod garden;
mod llm;

use std::time::{Duration, Instant};

use anyhow::Result;
use garden::{Action, Garden, BORDER, GRAVEL, RAKED};
use llm::Gardener;

const WIDTH: usize = 48;
const HEIGHT: usize = 20;
/// Session duration: 30 minutes per garden before automatic reset.
const SESSION_DURATION: Duration = Duration::from_secs(30 * 60);

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

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let model = std::env::var("OPENROUTER_MODEL")
        .unwrap_or_else(|_| "tencent/hy3:free".to_string());

    loop {
        let session_start = Instant::now();
        let mut garden = Garden::new(WIDTH, HEIGHT);
        let gardener = Gardener::new(&model, WIDTH, HEIGHT)?;
        let theme = gardener.theme_name().to_string();

        let mut consecutive_errors = 0;
        let mut border_drawn = false;
        let mut prompt_count: usize = 0;

        println!("\x1b[2J\x1b[H");
        println!("🌿 karesansui — Theme: \"{theme}\"\n");
        println!("   🐢 The turtle is waking up to tend the garden...\n");
        tokio::time::sleep(Duration::from_secs(2)).await;

        while session_start.elapsed() < SESSION_DURATION {
            let state = garden.render();
            let header = format!("🌿 karesansui — Theme: \"{theme}\"  [prompt #{prompt_count}]");
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

            // Skip duplicate draw_border calls.
            if matches!(action, Action::DrawBorder) && border_drawn {
                continue;
            }

            prompt_count += 1;
            let header = format!("🌿 karesansui — Theme: \"{theme}\"  [prompt #{prompt_count} — 🐢 building...]");

            match action {
                Action::DrawBorder => {
                    // Turtle walks around the perimeter laying bamboo frame.
                    for x in 0..WIDTH {
                        garden.grid[0][x] = BORDER.to_string();
                        garden.turtle_pos = Some((x, 0));
                        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }
                    for y in 0..HEIGHT {
                        garden.grid[y][WIDTH - 1] = BORDER.to_string();
                        garden.turtle_pos = Some((WIDTH - 1, y));
                        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }
                    for x in (0..WIDTH).rev() {
                        garden.grid[HEIGHT - 1][x] = BORDER.to_string();
                        garden.turtle_pos = Some((x, HEIGHT - 1));
                        print!("\x1b[2J\x1b[H{header}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }
                    for y in (0..HEIGHT).rev() {
                        garden.grid[y][0] = BORDER.to_string();
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
                Action::RakeLine { y, x1, x2 } => {
                    animate_walk(&mut garden, x1, y, &header).await?;
                    let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
                    let step_range: Vec<usize> = if x1 <= x2 {
                        (a..=b.min(WIDTH.saturating_sub(1))).collect()
                    } else {
                        (a..=b.min(WIDTH.saturating_sub(1))).rev().collect()
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
                Action::PlaceGravel { y, x1, x2 } => {
                    animate_walk(&mut garden, x1, y, &header).await?;
                    let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
                    let step_range: Vec<usize> = if x1 <= x2 {
                        (a..=b.min(WIDTH.saturating_sub(1))).collect()
                    } else {
                        (a..=b.min(WIDTH.saturating_sub(1))).rev().collect()
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
                    garden.turtle_glyph = "💤";
                    for remaining in (1..=20).rev() {
                        let h = format!("🌿 karesansui — \"{theme}\" — Complete! 💤 admiring ({remaining}s until reset)");
                        print!("\x1b[2J\x1b[H{h}\n\n{}", garden.render());
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    break;
                }
            }

            // Rate limiting & pacing between prompts:
            // Send prompt every 6 seconds, and after every 10 prompts wait 30 seconds.
            let wait_secs = if prompt_count % 10 == 0 { 30 } else { 6 };
            garden.turtle_glyph = "💤";
            for remaining in (1..=wait_secs).rev() {
                if session_start.elapsed() >= SESSION_DURATION {
                    break;
                }
                let status = if prompt_count % 10 == 0 {
                    format!("[prompt #{prompt_count} — 💤 resting 30s rate-limit pause: {remaining}s remaining]")
                } else {
                    format!("[prompt #{prompt_count} — 💤 resting: {remaining}s until next move]")
                };
                let h = format!("🌿 karesansui — Theme: \"{theme}\"  {status}");
                print!("\x1b[2J\x1b[H{h}\n\n{}", garden.render());
                std::io::Write::flush(&mut std::io::stdout())?;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            garden.turtle_glyph = "🐢";
        }

        println!("\x1b[2J\x1b[H");
        println!("🌿 karesansui — 30-minute garden cycle complete or finished.");
        println!("   🔄 Starting a brand new garden in 3 seconds...\n");
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
