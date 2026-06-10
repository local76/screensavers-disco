//! Consolidated disco screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).

pub mod types;
pub mod physics;

use library::core::{LcgRng, TerminalCell};
use std::time::Duration;
use library::core::screensaver::Screensaver;
use library::platform::native::sys_info::get_system_info;

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
    pub(crate) host_bias: f32,
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
            cpu_load: 0.4,
            host_bias,
        }
    }
}

impl Screensaver for Disco {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let delta = dt.as_secs_f32();
        self.time_elapsed += delta;

        // Live: system load drives the party intensity (pulses, more confetti)
        self.sys_refresh_timer += delta;
        if self.sys_refresh_timer >= 1.0 {
            let sys = get_system_info();
            self.mem_pressure = sys.mem_used_pct / 100.0;
            self.cpu_load = (self.mem_pressure * 0.6 + 0.3).min(0.9);
            if self.host_bias > 0.55 { self.cpu_load = (self.cpu_load + 0.1).min(0.98); }
            self.sys_refresh_timer = 0.0;
        }

        let audio_vol = self.audio_visualizer.get_volume();

        // OpenRGB audio visualizer updates
// Excite background stars randomly if audio is loud
        if audio_vol > 0.05 {
            for star in &mut self.stars {
                if self.rng.next_bool(0.0006 * audio_vol) {
                    star.excitation = self.rng.next_range(0.8, 1.4);
                    star.color = types::NEON_COLORS[self.rng.next_usize(types::NEON_COLORS.len())];
                }
            }
        }

        // Initialize confetti and stars if resized or not created
        if cols != self.last_cols || rows != self.last_rows {
            // Live load = bigger party
            let load_mult = 1.0 + self.cpu_load * 0.8;
            let target_confetti = (match self.confetti_density_opt {
                0 => 20,
                2 => 150,
                _ => 60,
            } as f32 * load_mult) as usize;
            self.confetti.clear();
            for _ in 0..target_confetti {
                let max_lt = self.rng.next_range(0.5, 2.0);
                self.confetti.push(types::Confetti {
                    x: 0.0,
                    y: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    color: types::NEON_COLORS[self.rng.next_usize(types::NEON_COLORS.len())],
                    ch: types::CONFETTI_CHARS[self.rng.next_usize(types::CONFETTI_CHARS.len())],
                    lifetime: 0.0, // force immediate respawn
                    max_lifetime: max_lt,
                });
            }

            self.stars.clear();
            let target_stars = (cols * rows / 20).clamp(20, 90);
            let disco_xf = cols as f32 / 2.0;
            let disco_yf = 4.0f32;

            for i in 0..target_stars {
                let sx = self.rng.next_f32();
                let sy = self.rng.next_f32();
                let star_xf = sx * cols as f32;
                let star_yf = sy * rows as f32;
                let dx = (star_xf - disco_xf) * 0.55;
                let dy = star_yf - disco_yf;
                let dist = (dx*dx + dy*dy).sqrt();
                let angle = if dy > 0.0 { dy.atan2(dx) } else { 0.0 };

                self.stars.push(types::Star {
                    x: sx,
                    y: sy,
                    phase: self.rng.next_f32() * std::f32::consts::TAU,
                    ch: if i % 8 == 0 { '✦' } else if i % 3 == 0 { '•' } else { '.' },
                    excitation: 0.0,
                    angle_to_disco: angle,
                    dist_to_disco: dist,
                    color: (255, 255, 255),
                });
            }

            self.last_cols = cols;
            self.last_rows = rows;
        }

        let disco_x = cols as f32 / 2.0;
        let disco_y = 4.0f32;

        // Decay star excitations
        for star in &mut self.stars {
            if star.excitation > 0.0 {
                star.excitation -= delta * 2.2;
                if star.excitation < 0.0 {
                    star.excitation = 0.0;
                }
            }
        }

        // Update confetti particles & excite stars
        let cols_f = cols as f32;
        let rows_f = rows as f32;
        for c in &mut self.confetti {
            c.lifetime -= delta;
            if c.lifetime <= 0.0 {
                let spawn_from_disco = self.rng.next_bool(0.4);
                let angle = self.rng.next_range(0.0, std::f32::consts::TAU);
                let speed = self.rng.next_range(10.0, 42.0) * (0.5 + 0.8 * audio_vol);
                
                if spawn_from_disco {
                    c.x = disco_x;
                    c.y = disco_y;
                } else {
                    c.x = cols as f32 / 2.0 + self.rng.next_range(-6.0, 6.0);
                    c.y = rows as f32 / 2.0 + self.rng.next_range(-2.0, 2.0);
                }
                c.vx = angle.cos() * speed / 0.55;
                c.vy = angle.sin() * speed;
                c.color = types::NEON_COLORS[self.rng.next_usize(types::NEON_COLORS.len())];
                c.ch = types::CONFETTI_CHARS[self.rng.next_usize(types::CONFETTI_CHARS.len())];
                c.max_lifetime = self.rng.next_range(0.6, 2.8);
                c.lifetime = c.max_lifetime;
            } else {
                c.x += c.vx * delta;
                c.y += c.vy * delta;
                c.vy += 1.2 * delta; // reduced gravity for wider, floaty arcs
            }

            // Confetti-star interaction
            for star in &mut self.stars {
                let sx = star.x * cols_f;
                let sy = star.y * rows_f;
                let dx = c.x - sx;
                let dy = (c.y - sy) * 2.0;
                let dist_sq = dx*dx + dy*dy;
                if dist_sq < 6.25 {
                    let dist = dist_sq.sqrt();
                    let force = (1.0 - dist / 2.5) * 1.5;
                    if force > star.excitation {
                        star.excitation = force;
                        star.color = c.color;
                    }
                }
            }
        }

        // Ray-star interaction
        if self.disco_ball_opt == 1 {
            let num_rays = 8;
            let base_angle = self.time_elapsed * 0.7;
            let max_ray_len = 35.0;
            for star in &mut self.stars {
                let mut best_ray_excitation = 0.0f32;
                let mut best_ray_color = (255, 255, 255);
                if star.dist_to_disco < max_ray_len && star.angle_to_disco > 0.0 {
                    for k in 0..num_rays {
                        let angle = base_angle + (k as f32) * (std::f32::consts::TAU / num_rays as f32);
                        let mut da = star.angle_to_disco - angle;
                        da = (da + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                        let spread = 0.14f32;
                        if da.abs() < spread {
                            let intensity = (1.0 - da.abs() / spread) * (1.0 - star.dist_to_disco / max_ray_len);
                            let force = intensity * 1.5;
                            if force > best_ray_excitation {
                                best_ray_excitation = force;
                                best_ray_color = types::NEON_COLORS[k % types::NEON_COLORS.len()];
                            }
                        }
                    }
                }
                if best_ray_excitation > star.excitation {
                    star.excitation = best_ray_excitation;
                    star.color = best_ray_color;
                }
            }
        }
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        physics::draw_disco(self, grid, cols, rows);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disco_creation() {
        let disco = Disco::new();
        assert_eq!(disco.time_elapsed, 0.0);
        assert_eq!(disco.last_cols, 0);
        assert_eq!(disco.last_rows, 0);
    }

    #[test]
    fn test_disco_update_and_draw() {
        let mut disco = Disco::new();
        let mut grid = vec![TerminalCell::default(); 80 * 24];
        
        // Initial update with duration and dimensions
        disco.update(Duration::from_millis(16), 80, 24);
        assert!(disco.time_elapsed > 0.0);
        assert_eq!(disco.last_cols, 80);
        assert_eq!(disco.last_rows, 24);
        
        // Draw to grid
        disco.draw(&mut grid, 80, 24);
        // Ensure some pixels are written or the grid was modified (background is black)
        assert_eq!(grid[0].bg, (0, 0, 0));
    }
}
