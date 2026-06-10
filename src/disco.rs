//! Consolidated disco screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).


use library::core::{LcgRng, TerminalCell};
use std::time::Duration;
use library::core::screensaver::Screensaver;

use library::platform::native::sys_info::get_system_info;

use library::toolkit::rgb_controller::{RgbController, is_openrgb_enabled};

use library::toolkit::rgb_protocol::RgbColor;
use library::core::logo_block::render_logo_block;

pub struct Confetti {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub color: (u8, u8, u8),
    pub ch: char,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

pub struct Star {
    pub x: f32,
    pub y: f32,
    pub phase: f32,
    pub ch: char,
    pub excitation: f32,
    pub angle_to_disco: f32,
    pub dist_to_disco: f32,
    pub color: (u8, u8, u8),
}

pub const NEON_COLORS: &[(u8, u8, u8)] = &[
    (255, 0, 128),  // Neon Pink
    (0, 255, 255),  // Neon Cyan
    (255, 255, 0),  // Neon Yellow
    (50, 255, 50),  // Neon Green
    (180, 0, 255),  // Neon Purple
    (255, 127, 0),  // Neon Orange
];

pub const CONFETTI_CHARS: &[char] = &['*', '+', 'o', 'x', '•'];


pub struct Disco {
    rng: LcgRng,
    pub(crate) confetti: Vec<Confetti>,
    pub(crate) stars: Vec<Star>,
    pub(crate) time_elapsed: f32,
    pub(crate) last_cols: usize,
    pub(crate) last_rows: usize,
    pub(crate) audio_visualizer: self::audio::AudioVisualizer,
    confetti_density_opt: u32,
    pub(crate) disco_ball_opt: u32,

    // Live system dynamics
    sys_refresh_timer: f32,
    mem_pressure: f32,
    cpu_load: f32,
    host_bias: f32,
    rgb: Option<RgbController>,
    rgb_timer: f32,
}

impl Default for Disco {
    fn default() -> Self {
        Self::new()
    }
}

impl Disco {
    pub fn new() -> Self {
        // Pre-4.1 HKEY_CURRENT_USER registry reads (ConfettiDensity, DiscoBall)
        // collapsed to defaults for the inline migration. Re-added in 4.2.
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
            audio_visualizer: self::audio::AudioVisualizer::new(),
            confetti_density_opt,
            disco_ball_opt,
            sys_refresh_timer: 0.0,
            mem_pressure: sys.mem_used_pct / 100.0,
            cpu_load: 0.4,
            host_bias,
            rgb: if is_openrgb_enabled() { Some(RgbController::new()) } else { None },
            rgb_timer: 0.0,
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
        self.rgb_timer += delta;
        if self.rgb_timer >= 0.06 {
            self.rgb_timer = 0.0;
            if let Some(ref r) = self.rgb {

                // Beat detection: trigger a flash if there's a loud beat
                if audio_vol > 0.4 {
                    let color_idx = self.rng.next_usize(NEON_COLORS.len());
                    let base_c = NEON_COLORS[color_idx];
                    let flash_color = RgbColor::new(base_c.0, base_c.1, base_c.2);
                    r.flash(flash_color, Duration::from_millis(150));
                } else {
                    // Continuous visualizer: cycle neon colors modulated by volume
                    let vol_factor = (audio_vol * 1.5 + 0.15).clamp(0.15, 1.0);
                    let get_party_color = |offset: usize, time_scale: f32| -> RgbColor {
                        let color_idx = ((self.time_elapsed * time_scale) as usize + offset) % NEON_COLORS.len();
                        let base_c = NEON_COLORS[color_idx];
                        RgbColor::new(
                            (base_c.0 as f32 * vol_factor) as u8,
                            (base_c.1 as f32 * vol_factor) as u8,
                            (base_c.2 as f32 * vol_factor) as u8,
                        )
                    };
                    
                    r.set_device_color(5, get_party_color(0, 3.0));
                    r.set_device_color(6, get_party_color(1, 3.0));
                    r.set_device_color(12, get_party_color(2, 3.0));
                    let c_internal = get_party_color(3, 3.0);
                    r.set_device_color(0, c_internal);
                    r.set_device_color(1, c_internal);
                    r.set_device_color(2, c_internal);
                }
            }
        }

        // Excite background stars randomly if audio is loud
        if audio_vol > 0.05 {
            for star in &mut self.stars {
                if self.rng.next_bool(0.0006 * audio_vol) {
                    star.excitation = self.rng.next_range(0.8, 1.4);
                    star.color = NEON_COLORS[self.rng.next_usize(NEON_COLORS.len())];
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
                self.confetti.push(Confetti {
                    x: 0.0,
                    y: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    color: NEON_COLORS[self.rng.next_usize(NEON_COLORS.len())],
                    ch: CONFETTI_CHARS[self.rng.next_usize(CONFETTI_CHARS.len())],
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

                self.stars.push(Star {
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
                c.color = NEON_COLORS[self.rng.next_usize(NEON_COLORS.len())];
                c.ch = CONFETTI_CHARS[self.rng.next_usize(CONFETTI_CHARS.len())];
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
                                best_ray_color = NEON_COLORS[k % NEON_COLORS.len()];
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
        self.draw_impl(grid, cols, rows);
    }
}


impl Disco {
    pub fn draw_impl(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        if cols == 0 || rows == 0 {
            return;
        }
        let disco_x = cols / 2;
        let disco_y = 4usize;

        // Apply solid black background (no flashes)
        for cell in grid.iter_mut() {
            cell.bg = (0, 0, 0);
        }

        // Find top candidates for lens flares (only highly excited stars, max 4)
        let mut flare_candidates: Vec<(usize, f32)> = self.stars.iter()
            .enumerate()
            .filter(|(_, star)| star.excitation > 0.8)
            .map(|(idx, star)| (idx, star.excitation))
            .collect();
        flare_candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
        let allowed_flares: Vec<usize> = flare_candidates.iter()
            .take(4)
            .map(|&(idx, _)| idx)
            .collect();

        // 2. Render background stars (excited by confetti and disco light rays)
        let num_rays = 8;
        let base_angle = self.time_elapsed * 0.7;
        let max_ray_len = 35.0;

        for (i, star) in self.stars.iter().enumerate() {
            let sx = (star.x * cols as f32) as usize;
            let sy = (star.y * rows as f32) as usize;

            if sx < cols && sy < rows {
                let sparkle_base = ((self.time_elapsed * 2.0 + star.phase).sin() + 1.0) * 0.5;
                let sparkle = (sparkle_base + star.excitation).min(2.0);

                let mut r = (50.0 + sparkle * 80.0) as u8;
                let mut g = (50.0 + sparkle * 80.0) as u8;
                let mut b = (65.0 + sparkle * 75.0) as u8;

                if star.excitation > 0.05 {
                    let blend = (star.excitation * 0.6).min(1.0);
                    r = (r as f32 * (1.0 - blend) + star.color.0 as f32 * blend).min(255.0) as u8;
                    g = (g as f32 * (1.0 - blend) + star.color.1 as f32 * blend).min(255.0) as u8;
                    b = (b as f32 * (1.0 - blend) + star.color.2 as f32 * blend).min(255.0) as u8;
                }

                let ch = if sparkle > 1.2 {
                    '✹'
                } else if sparkle > 0.6 {
                    '✦'
                } else {
                    star.ch
                };

                grid[sy * cols + sx] = TerminalCell {
                    ch,
                    fg: (r, g, b),
                    bg: (0, 0, 0),
                    bold: sparkle > 0.6 || star.excitation > 0.3,
                };

                // Draw lens flares and starbursts on highly excited stars
                if allowed_flares.contains(&i) {
                    let flare_intensity = ((star.excitation - 0.8) / 0.7 + 0.5).min(1.5);
                    let flare_color = star.color;

                    // Horizontal streak
                    let h_len = 12;
                    for dx in 1..h_len {
                        let alpha = (100.0 * flare_intensity).max(20.0) as u8;
                        let fade = alpha.saturating_sub((dx * (90 / h_len)) as u8);
                        if fade > 8 {
                            if sx + dx < cols {
                                let cell = &mut grid[sy * cols + (sx + dx)];
                                if cell.ch == ' ' || cell.ch == '─' {
                                    cell.ch = '─';
                                    cell.fg = (
                                        (fade as f32 * 0.4 + flare_color.0 as f32 * 0.6) as u8,
                                        (fade as f32 * 0.3 + flare_color.1 as f32 * 0.6) as u8,
                                        (fade.saturating_add(30) as f32 * 0.4 + flare_color.2 as f32 * 0.6) as u8,
                                    );
                                }
                            }
                            if sx >= dx {
                                let cell = &mut grid[sy * cols + (sx - dx)];
                                if cell.ch == ' ' || cell.ch == '─' {
                                    cell.ch = '─';
                                    cell.fg = (
                                        (fade as f32 * 0.4 + flare_color.0 as f32 * 0.6) as u8,
                                        (fade as f32 * 0.3 + flare_color.1 as f32 * 0.6) as u8,
                                        (fade.saturating_add(30) as f32 * 0.4 + flare_color.2 as f32 * 0.6) as u8,
                                    );
                                }
                            }
                        }
                    }

                    // Vertical streak
                    let v_len = 4;
                    for dy in 1..v_len {
                        let alpha = (80.0 * flare_intensity).max(15.0) as u8;
                        let fade = alpha.saturating_sub((dy * (70 / v_len)) as u8);
                        if fade > 8 {
                            if sy + dy < rows {
                                let cell = &mut grid[(sy + dy) * cols + sx];
                                if cell.ch == ' ' || cell.ch == '│' {
                                    cell.ch = '│';
                                    cell.fg = (
                                        (fade as f32 * 0.4 + flare_color.0 as f32 * 0.6) as u8,
                                        (fade as f32 * 0.3 + flare_color.1 as f32 * 0.6) as u8,
                                        (fade.saturating_add(20) as f32 * 0.4 + flare_color.2 as f32 * 0.6) as u8,
                                    );
                                }
                            }
                            if sy >= dy {
                                let cell = &mut grid[(sy - dy) * cols + sx];
                                if cell.ch == ' ' || cell.ch == '│' {
                                    cell.ch = '│';
                                    cell.fg = (
                                        (fade as f32 * 0.4 + flare_color.0 as f32 * 0.6) as u8,
                                        (fade as f32 * 0.3 + flare_color.1 as f32 * 0.6) as u8,
                                        (fade.saturating_add(20) as f32 * 0.4 + flare_color.2 as f32 * 0.6) as u8,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        if self.disco_ball_opt == 1 {
            // 3. Render Disco Ball light rays sweeping
            for k in 0..num_rays {
                let angle = base_angle + (k as f32) * (std::f32::consts::TAU / num_rays as f32);
                let ray_color_base = NEON_COLORS[k % NEON_COLORS.len()];
                let cos_val = angle.cos() / 0.55;
                let sin_val = angle.sin();

                for d in 4..30 {
                    let rx = disco_x as f32 + d as f32 * cos_val;
                    let ry = disco_y as f32 + d as f32 * sin_val;

                    if rx >= 0.0 && rx < cols as f32 && ry >= 0.0 && ry < rows as f32 {
                        let x = rx as usize;
                        let y = ry as usize;
                        let intensity = 1.0 - (d as f32 / max_ray_len).min(1.0);
                        if intensity > 0.05 {
                            let ch = if d < 10 { '*' } else if d < 20 { '+' } else { '.' };
                            let color = (
                                (ray_color_base.0 as f32 * intensity) as u8,
                                (ray_color_base.1 as f32 * intensity) as u8,
                                (ray_color_base.2 as f32 * intensity) as u8,
                            );
                            let idx = y * cols + x;
                            
                            let current_ch = grid[idx].ch;
                            if current_ch == ' ' || current_ch == '─' || current_ch == '│' || current_ch == '+' || current_ch == '.' || current_ch == '*' {
                                grid[idx].ch = ch;
                                grid[idx].fg = color;
                            }
                        }
                    }
                }
            }

            // 4. Render Shimmering Disco Ball itself
            let radius_y = 3i32;
            let radius_x = 5i32;
            let disc_chars = ['░', '▒', '▓', '█'];
            let rot_offset = (self.time_elapsed * 5.0) as i32;

            for dy in -radius_y..=radius_y {
                let width_at_y = (radius_x as f32 * (1.0 - (dy as f32 / radius_y as f32).powi(2)).max(0.0).sqrt()).round() as i32;
                for dx in -width_at_y..=width_at_y {
                    let sx = (disco_x as i32 + dx) as usize;
                    let sy = (disco_y as i32 + dy) as usize;
                    if sx < cols && sy < rows {
                        let pattern_idx = ((dx + dy + rot_offset).abs() % 4) as usize;
                        let ch = disc_chars[pattern_idx];
                        let dist_center = (dx as f32).powi(2) + (dy as f32 * 1.8).powi(2);
                        let fg = if dist_center < 3.0 {
                            (255, 255, 255) // White hotspot
                        } else if dist_center < 9.0 {
                            (200, 220, 255) // Bright Silver
                        } else {
                            (100, 110, 130) // Silver/Gray
                        };
                        let idx = sy * cols + sx;
                        grid[idx].ch = ch;
                        grid[idx].fg = fg;
                    }
                }
            }

            // Draw disco string / hanger
            for sy in 0..(disco_y.saturating_sub(radius_y as usize)) {
                let idx = sy * cols + disco_x;
                grid[idx].ch = '│';
                grid[idx].fg = (100, 100, 100);
            }
        }

        // 5. Render Bouncing Equalizer Bars (bottom)
        let audio_vol = self.audio_visualizer.get_volume();
        let max_bar_h = (rows / 5).clamp(4, 10);
        let bar_spacing = 3;
        let mut rng_eq = LcgRng::new(999);
        for x in (0..cols).step_by(bar_spacing) {
            let val = if audio_vol > 0.005 {
                let freq1 = 2.4;
                let freq2 = 5.6;
                let phase = x as f32 * 0.15;
                let sin_v1 = (self.time_elapsed * freq1 + phase).sin().abs();
                let sin_v2 = (self.time_elapsed * freq2 - phase * 0.5).cos().abs() * 0.4;
                let rand_noise = rng_eq.next_f32() * 0.25;
                let raw_val = (sin_v1 + sin_v2 + rand_noise).clamp(0.0, 1.0);
                
                let scale = (audio_vol * 1.5).min(1.0);
                raw_val * scale
            } else {
                0.0
            };
            let bar_h = (val * max_bar_h as f32).round() as usize;

            for dy in 0..bar_h {
                let y = rows.saturating_sub(1).saturating_sub(dy);
                if y >= rows { continue; }
                let pct = dy as f32 / max_bar_h as f32;
                let color = if pct < 0.4 {
                    (0, 255, 128) // Bottom green-cyan
                } else if pct < 0.75 {
                    (255, 200, 0) // Middle yellow-orange
                } else {
                    (255, 0, 100) // Top red-pink
                };

                for dx in 0..2 {
                    let bx = x + dx;
                    if bx < cols {
                        let idx = y * cols + bx;
                        grid[idx].ch = '█';
                        grid[idx].fg = color;
                    }
                }
            }
        }

        // 6. Exploding Neon Confetti Particles
        for c in &self.confetti {
            let px = c.x as usize;
            let py = c.y as usize;
            if px < cols && py < rows {
                let life_pct = c.lifetime / c.max_lifetime;
                let col = (
                    (c.color.0 as f32 * life_pct) as u8,
                    (c.color.1 as f32 * life_pct) as u8,
                    (c.color.2 as f32 * life_pct) as u8,
                );
                let idx = py * cols + px;
                let current_ch = grid[idx].ch;
                if current_ch == ' ' || current_ch == '─' || current_ch == '│' || current_ch == '+' || current_ch == '.' || current_ch == '*' {
                    grid[idx].ch = c.ch;
                    grid[idx].fg = col;
                }
            }
        }

        // library 4.1: render the centered system logo from the live OS info
        // (replaces pre-4.1 `trance_core::logo_lines()` + `logo_dimensions()`).
        let logo_text = get_system_info().logo_text;
        let lines = render_logo_block(&logo_text, None);
        let logo_h = lines.len();
        let logo_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let logo_x = cols.saturating_sub(logo_w) / 2;
        let logo_y = rows.saturating_sub(logo_h) / 2;

        if logo_x < cols && logo_y < rows {
            let r = ((self.time_elapsed * 4.5).sin() * 127.0 + 128.0) as u8;
            let g = ((self.time_elapsed * 4.5 + 2.0).sin() * 127.0 + 128.0) as u8;
            let b = ((self.time_elapsed * 4.5 + 4.0).sin() * 127.0 + 128.0) as u8;
            let logo_color = (r, g, b);

            for (r_offset, line) in lines.iter().enumerate().take(logo_h) {
                let gy = logo_y + r_offset;
                if gy >= rows { continue; }
                for (c_offset, ch) in line.chars().enumerate() {
                    let gx = logo_x + c_offset;
                    if gx >= cols { continue; }
                    if ch != ' ' {
                        let idx = gy * cols + gx;
                        grid[idx].ch = ch;
                        grid[idx].fg = logo_color;
                        grid[idx].bold = true;
                    }
                }
            }
        }
    }
}


pub mod audio {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::thread;
    use std::time::Duration;
    
    /// Beat-driven audio visualizer. Reads the system audio mix on Windows via
    /// the `IAudioClient` WASAPI loopback API, and falls back to a sine+noise
    /// generator on other platforms (and on the 4.1 inline migration while
    /// the Windows-specific capture routine is being stabilized for the
    /// `windows` crate v0.52 API — see library CHANGELOG 4.1.7).
    pub struct AudioVisualizer {
        volume: Arc<AtomicU32>,
        should_stop: Arc<AtomicBool>,
    }
    
    impl AudioVisualizer {
        pub fn new() -> Self {
            let volume = Arc::new(AtomicU32::new(0));
            let should_stop = Arc::new(AtomicBool::new(false));
    
            let volume_clone = volume.clone();
            let should_stop_clone = should_stop.clone();
    
            // library 4.1.7 inline-migration: the pre-4.1.7 disco Windows
            // audio capture routine (`unsafe fn run_audio_capture` using
            // IAudioClient loopback) is preserved as a comment in the
            // library CHANGELOG but not compiled in 4.1.7. The same
            // sine+noise fallback the non-Windows path uses produces the
            // visualizer's beat pattern on Windows too. The real WASAPI
            // capture returns in 4.2 once the windows v0.52 API shift
            // (single-arg `Activate`, three-arg `CoCreateInstance`) is
            // fully verified against the live disco screensaver preview.
            thread::spawn(move || {
                let mut angle = 0.0f32;
                let mut seed = 12345u32;
                while !should_stop_clone.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(15));
                    angle += 0.1;
                    seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                    let rand_val = ((seed / 65536) % 32768) as f32 / 32768.0;
    
                    let base = (angle.sin().abs() * 0.4) + 0.1;
                    let noise = rand_val * 0.2;
                    let vol_f32 = (base + noise).min(1.0);
                    let vol_u32 = (vol_f32 * 1000.0) as u32;
                    volume_clone.store(vol_u32, Ordering::SeqCst);
                }
            });
    
            Self { volume, should_stop }
        }
    
        pub fn get_volume(&self) -> f32 {
            self.volume.load(Ordering::SeqCst) as f32 / 1000.0
        }
    }
    
    impl Drop for AudioVisualizer {
        fn drop(&mut self) {
            self.should_stop.store(true, Ordering::SeqCst);
        }
    }
    
}
