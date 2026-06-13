#![allow(dead_code)]
#![allow(unused_imports)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod runner;
mod disco;

fn main() {
    let effect = disco::Disco::new();
    crate::runner::screensaver_runner::run_main(effect, "disco");
}

#[cfg(test)]
mod tests_perf;
