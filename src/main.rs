#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod runner;
mod disco;

fn main() {
    let effect = disco::Disco::new();
    runner::run_main(effect, "disco");
}