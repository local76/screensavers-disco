use crate::runner::core::screensaver::Screensaver;
use crate::runner::core::TerminalCell;
use crate::disco::types;
use crate::disco::physics;
use crate::disco::Disco;
use std::time::Duration;

impl Screensaver for Disco {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let dt_secs = dt.as_secs_f32();

        // Auto-detect high refresh rates during the startup phase
        if self.time_elapsed < 2.0 && dt_secs > 0.001 {
            if dt_secs < self.target_frame_time - 0.001 {
                self.target_frame_time = dt_secs;
            }
        }

        // Exponential moving average for frame time (alpha = 0.1)
        self.frame_time_ema = self.frame_time_ema * 0.9 + dt_secs.min(0.2) * 0.1;

        let speed_mult = if self.on_battery { 0.65 } else { 1.0 };
        let delta = dt_secs * speed_mult;
        self.time_elapsed += delta;

        // Adjust quality_scale based on frame time performance vs target
        if self.time_elapsed > 1.5 {
            if self.frame_time_ema > self.target_frame_time * 1.15 {
                self.quality_scale = (self.quality_scale - 0.15 * delta).max(0.20);
            } else if self.frame_time_ema < self.target_frame_time * 1.05 {
                self.quality_scale = (self.quality_scale + 0.04 * delta).min(1.0);
            }
        }

        // Live: system load drives the party intensity (pulses, more confetti)
        self.sys_refresh_timer += delta;
        if self.sys_refresh_timer >= 1.0 {
            let sys = crate::runner::toolkit::sys_info::get_system_info();
            self.mem_pressure = sys.mem_used_pct / 100.0;
            self.cpu_load = (sys.cpu_usage_pct / 100.0).clamp(0.0, 1.0);
            self.on_battery = sys.power_status.contains("Battery");
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

        // Initialize grid size trackers if resized
        if cols != self.last_cols || rows != self.last_rows {
            self.confetti.clear();
            self.stars.clear();
            self.last_cols = cols;
            self.last_rows = rows;
        }

        // Live load = bigger party
        let load_mult = 1.0 + self.cpu_load * 0.8;
        let target_confetti = ((match self.confetti_density_opt {
            0 => 20,
            2 => 150,
            _ => 60,
        } as f32 * load_mult) * self.quality_scale * (if self.on_battery { 0.55 } else { 1.0 })) as usize;

        if self.confetti.len() > target_confetti {
            self.confetti.truncate(target_confetti);
        } else if self.confetti.len() < target_confetti && target_confetti > 0 {
            while self.confetti.len() < target_confetti {
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
        }

        let target_stars = (((cols * rows / 20).clamp(20, 90)) as f32 * self.quality_scale * (if self.on_battery { 0.55 } else { 1.0 })) as usize;
        let disco_xf = cols as f32 / 2.0;
        let disco_yf = 4.0f32;

        if self.stars.len() > target_stars {
            self.stars.truncate(target_stars);
        } else if self.stars.len() < target_stars && target_stars > 0 {
            while self.stars.len() < target_stars {
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
                    ch: if self.stars.len() % 8 == 0 { '✦' } else if self.stars.len() % 3 == 0 { '•' } else { '.' },
                    excitation: 0.0,
                    angle_to_disco: angle,
                    dist_to_disco: dist,
                    color: (255, 255, 255),
                });
            }
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
