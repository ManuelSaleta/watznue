use std::io::{stdout, IsTerminal, Write};
use std::time::Duration;
use tokio::time::sleep;

const FRAME_1: &str = include_str!("../../../docs/images/watznue-catbot.ascii");
const FRAME_2: &str = include_str!("../../../docs/images/watznue-catbot-frame2.ascii");

#[tokio::main]
async fn main() {
    let mut out = stdout();
    if !out.is_terminal() {
        println!("Not a terminal; skipping mascot animation.");
        return;
    }

    let f1_lines: Vec<&str> = FRAME_1.lines().collect();
    let f2_lines: Vec<&str> = FRAME_2.lines().collect();
    let total_lines = f1_lines.len();

    println!("\x1b[?25l"); // Hide cursor

    let stages = [
        "[1/3] Detecting pending packages via DNF5...",
        "[2/3] Sniffing Bodhi advisories & CVE metadata [14/42]...",
        "[3/3] Normalizing changesets & classifying tiers...",
    ];

    for i in 0..6 {
        let frame = if i % 2 == 0 { &f1_lines } else { &f2_lines };
        let stage = stages[std::cmp::min(i / 2, stages.len() - 1)];

        if i > 0 {
            print!("\x1b[{}A\r", total_lines + 2);
        }

        println!("\x1b[2K\x1b[1;34m watznue v0.1.0\x1b[0m \x1b[2m— What's new?\x1b[0m");
        println!("\x1b[2K\x1b[1;33m {}\x1b[0m", stage);

        for line in frame {
            println!("\x1b[2K{}", line);
        }
        let _ = out.flush();

        sleep(Duration::from_millis(350)).await;
    }

    // Ephemeral cleanup: wipe lines
    print!("\x1b[{}A\r", total_lines + 2);
    for _ in 0..(total_lines + 2) {
        println!("\x1b[2K");
    }
    print!("\x1b[{}A\r", total_lines + 2);
    println!("\x1b[?25h\x1b[1;32m✔ Completed!\x1b[0m Found 42 package(s) with pending updates.\n");
    let _ = out.flush();
}
