//! CLI flags (parsed by hand — no deps, keeps the binary small).

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: String,
    pub port: u16,
    pub interval_ms: u64,
    pub history_ticks: usize,
    pub sys_root: PathBuf,
    pub static_dir: Option<PathBuf>,
    pub token: Option<String>,
    pub smartctl: bool,
}

const USAGE: &str = "Usage: ryzenai-dashboard [options]

Options:
  --bind <addr>         listen address (default: 127.0.0.1)
  --port <n>            TCP port (default: 3010)
  --interval <ms>       collection interval (default: 2000)
  --history <ticks>     ring buffer length per series (default: 900)
  --sys-root <path>     sysfs/procfs root, dev use only (default: /)
  --static <dir>        serve UI from a directory instead of the embedded UI
  --token <t>           require a Bearer token on /api/* (query token or Authorization header)
  --smartctl            enable optional smartctl probing (needs privileges)
  -h, --help            show this help
  -V, --version         show version
";

pub fn parse() -> Config {
    let mut c = Config {
        bind_addr: "127.0.0.1".into(),
        port: 3010,
        interval_ms: 2000,
        history_ticks: 900,
        sys_root: PathBuf::from("/"),
        static_dir: None,
        token: None,
        smartctl: false,
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].clone();
        match a.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("ryzenai-dashboard {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--bind" => c.bind_addr = value(&args, &mut i, "--bind"),
            "--port" => c.port = value(&args, &mut i, "--port").parse().unwrap_or_else(|_| die("--port")),
            "--interval" => c.interval_ms = value(&args, &mut i, "--interval").parse().unwrap_or_else(|_| die("--interval")),
            "--history" => c.history_ticks = value(&args, &mut i, "--history").parse().unwrap_or_else(|_| die("--history")),
            "--sys-root" => c.sys_root = PathBuf::from(value(&args, &mut i, "--sys-root")),
            "--static" => c.static_dir = Some(PathBuf::from(value(&args, &mut i, "--static"))),
            "--token" => c.token = Some(value(&args, &mut i, "--token")),
            "--smartctl" => c.smartctl = true,
            other => {
                if other.starts_with("--") {
                    eprintln!("error: unknown flag: {other}");
                    print!("{USAGE}");
                    std::process::exit(1);
                }
            }
        }
        i += 1;
    }
    c.interval_ms = c.interval_ms.clamp(200, 60_000);
    c.history_ticks = c.history_ticks.clamp(32, 7_200);
    c
}

fn value(args: &[String], i: &mut usize, flag: &str) -> String {
    let idx = *i + 1;
    if idx >= args.len() {
        die(flag);
    }
    *i = idx;
    args[idx].clone()
}

fn die(flag: &str) -> ! {
    eprintln!("error: {flag} requires a value");
    print!("{USAGE}");
    std::process::exit(1);
}