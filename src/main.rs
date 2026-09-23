// No console window behind the game on Windows. The flags still need
// somewhere to print, so `attach_console` borrows the terminal the game was
// started from, if there was one.
#![cfg_attr(windows, windows_subsystem = "windows")]

use gorillas::game::Game;
use gorillas::qb::Qb;

struct Config {
    seed: u64,
    speed: f64,
    mute: bool,
    scale: usize,
}

fn parse_args() -> Config {
    let mut cfg = Config {
        seed: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(1),
        speed: 1.0,
        mute: false,
        scale: 2,
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" => {
                i += 1;
                cfg.seed = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(cfg.seed);
            }
            "--speed" => {
                i += 1;
                // rest() divides by this, so zero clamps to the one hour
                // ceiling and looks exactly like a hang.
                cfg.speed = args
                    .get(i)
                    .and_then(|s| s.parse().ok())
                    .filter(|f: &f64| f.is_finite() && *f > 0.0)
                    .unwrap_or(1.0);
            }
            "--scale" => {
                i += 1;
                cfg.scale = args
                    .get(i)
                    .and_then(|s| s.parse().ok())
                    .filter(|n| *n > 0)
                    .unwrap_or(2);
            }
            "--mute" => cfg.mute = true,
            "--version" | "-V" => {
                println!("gorilla-rust {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--help" | "-h" => {
                println!(
                    "gorilla-rust [--seed N] [--speed F] [--scale N] [--mute]\n\n\
                     --seed   fix the random seed so a run repeats\n\
                     --speed  scale how fast the game runs, 1.0 is normal,\n\
                     \x20        2.0 halves every delay\n\
                     --scale  window scale factor, 2 is the default\n\
                     --mute   turn off sound\n\
                     --version, -V   print the version and exit"
                );
                std::process::exit(0);
            }
            other => eprintln!("ignoring unknown argument {other}"),
        }
        i += 1;
    }
    cfg
}

/// On Windows the game has no console of its own. When it was started from
/// a terminal, attach to that one so `--help`, `--version` and warnings show
/// up there. Double-clicked, there is no parent console and this does
/// nothing.
fn attach_console() {
    #[cfg(windows)]
    // SAFETY: AttachConsole takes a plain process id and has no memory
    // preconditions; failure (no parent console) is harmless and ignored.
    unsafe {
        use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

fn main() {
    attach_console();
    let cfg = parse_args();
    let mut qb = Qb::windowed(640, 350, cfg.scale);
    qb.speed = cfg.speed;
    if cfg.mute {
        qb.audio = gorillas::qb::backend::Audio::new(true);
    }
    let mut game = Game::new(qb, cfg.seed);
    // A closed window unwinds out of the whole game, which is not an error.
    let _ = pollster::block_on(game.run());
}
