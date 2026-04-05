// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let force_logout = args.iter().any(|a| a == "--logout");
    chuck_lib::run(force_logout);
}
