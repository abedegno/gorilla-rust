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

fn main() {
    let cfg = parse_args();
    let mut qb = Qb::windowed(640, 350, cfg.scale);
    qb.speed = cfg.speed;
    if cfg.mute {
        qb.audio = gorillas::qb::sound::Audio::new(true);
    }
    let mut game = Game::new(qb, cfg.seed);
    // A closed window unwinds out of the whole game, which is not an error.
    let _ = game.run();
}
