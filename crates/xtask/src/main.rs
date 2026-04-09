use std::env;
use std::process;

mod bump;
mod changelog;
mod check;
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
            if let Err(e) = changelog::run() {
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
            eprintln!("  changelog                  Generate CHANGELOG.md from git history");
            process::exit(1);
        }
    }
}
