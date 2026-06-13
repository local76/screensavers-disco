//! Consolidated disco screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).

pub mod types;
pub mod physics;
pub mod render_stars;
pub mod render_ball_eq_logo;
pub mod effect;

use crate::runner::core::LcgRng;
use crate::runner::toolkit::sys_info::get_system_info;

pub struct Disco {
    pub(crate) rng: LcgRng,
    pub(crate) confetti: Vec<types::Confetti>,
    pub(crate) stars: Vec<types::Star>,
    pub(crate) time_elapsed: f32,
    pub(crate) last_cols: usize,
    pub(crate) last_rows: usize,
    pub(crate) audio_visualizer: types::audio::AudioVisualizer,
    pub(crate) confetti_density_opt: u32,
    pub(crate) disco_ball_opt: u32,

    // Live system dynamics
    pub(crate) sys_refresh_timer: f32,
    pub(crate) mem_pressure: f32,
    pub(crate) cpu_load: f32,
    pub(crate) _host_bias: f32,
    pub(super) on_battery: bool,
    pub(super) frame_time_ema: f32,
    pub(super) quality_scale: f32,
    pub(super) target_frame_time: f32,
}

impl Default for Disco {
    fn default() -> Self {
        Self::new()
    }
}

impl Disco {
    pub fn new() -> Self {
        let confetti_density_opt: u32 = 1;
        let disco_ball_opt: u32 = 1;

        let sys = get_system_info();
        let host_bias = sys.hostname.chars().map(|c| c as u32).sum::<u32>() as f32 / 1000.0 % 1.0;
        let on_battery = sys.power_status.contains("Battery");

        Self {
            rng: LcgRng::new(4321),
            confetti: Vec::new(),
            stars: Vec::new(),
            time_elapsed: 0.0,
            last_cols: 0,
            last_rows: 0,
            audio_visualizer: types::audio::AudioVisualizer::new(),
            confetti_density_opt,
            disco_ball_opt,
            sys_refresh_timer: 0.0,
            mem_pressure: sys.mem_used_pct / 100.0,
            cpu_load: (sys.cpu_usage_pct / 100.0).clamp(0.0, 1.0),
            _host_bias: host_bias,
            on_battery,
            frame_time_ema: 0.01666667,
            quality_scale: 1.0,
            target_frame_time: 0.01666667,
        }
    }
}

#[cfg(test)]
#[path = "disco_tests.rs"]
mod tests;
