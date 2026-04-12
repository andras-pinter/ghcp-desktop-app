use std::env;
use std::process;

mod bump;
mod changelog;
mod check;
mod release;
mod version;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str);

    match cmd {
        Some("bump") => {
            let level = args.get(1).map(String::as_str);
            match level {
                Some("major") | Some("minor") | Some("patch") => {
                    if let Err(e) = bump::run(level.unwrap()) {
                        eprintln!("Error: {e}");
                        process::exit(1);
                    }
                }
                _ => {
                    eprintln!("Usage: cargo xtask bump <major|minor|patch>");
                    process::exit(1);
                }
            }
        }
        Some("check-version") => {
            if let Err(e) = check::run() {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
        Some("changelog") => {
            let since = args.get(1).and_then(|a| {
                if a == "--since" {
                    args.get(2).map(String::as_str)
                } else {
                    None
                }
            });
            if let Err(e) = changelog::run(since) {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
        Some("release") => {
            let mut force_level = None;
            let mut dry_run = false;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--dry-run" => dry_run = true,
                    "--bump" => {
                        i += 1;
                        match args.get(i).map(String::as_str) {
                            Some("major") | Some("minor") | Some("patch") => {
                                force_level = args.get(i).map(String::as_str);
                            }
                            Some(other) => {
                                eprintln!("Invalid bump level: {other}");
                                eprintln!("Expected: major, minor, or patch");
                                process::exit(1);
                            }
                            None => {
                                eprintln!("--bump requires a value: major, minor, or patch");
                                process::exit(1);
                            }
                        }
                    }
                    other => {
                        eprintln!("Unknown option for release: {other}");
                        process::exit(1);
                    }
                }
                i += 1;
            }
            if let Err(e) = release::run(force_level, dry_run) {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage: cargo xtask <command>");
            eprintln!();
            eprintln!("Commands:");
            eprintln!("  bump <major|minor|patch>   Bump version across all project files");
            eprintln!("  check-version              Verify all version strings are in sync");
            eprintln!("  changelog [--since <tag>]   Generate CHANGELOG.md from git history");
            eprintln!("  release [--dry-run] [--bump <level>]  Auto-detect and release");
            process::exit(1);
        }
    }
}
