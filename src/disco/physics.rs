use library::core::TerminalCell;
use library::core::LcgRng;
use library::toolkit::sys_info::get_system_info;
use library::core::logo_block::render_logo_block;

use crate::disco::types::NEON_COLORS;
use crate::disco::Disco;

pub fn draw_disco(disco: &Disco, grid: &mut [TerminalCell], cols: usize, rows: usize) {
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
    let mut flare_candidates: Vec<(usize, f32)> = disco.stars.iter()
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
    let base_angle = disco.time_elapsed * 0.7;
    let max_ray_len = 35.0;

    for (i, star) in disco.stars.iter().enumerate() {
        let sx = (star.x * cols as f32) as usize;
        let sy = (star.y * rows as f32) as usize;

        if sx < cols && sy < rows {
            let sparkle_base = ((disco.time_elapsed * 2.0 + star.phase).sin() + 1.0) * 0.5;
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

    if disco.disco_ball_opt == 1 {
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
        let rot_offset = (disco.time_elapsed * 5.0) as i32;

        for dy in -radius_y..=radius_y {
            let width_at_y = (radius_x as f32 * (1.0 - (dy as f32 / radius_y as f32).powi(2)).max(0.0).sqrt()).round() as i32;
            for dx in -width_at_y..=width_at_y {
                let sx_i = disco_x as i32 + dx;
                let sy_i = disco_y as i32 + dy;
                if sx_i >= 0 && sy_i >= 0 {
                    let sx = sx_i as usize;
                    let sy = sy_i as usize;
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
        }

        // Draw disco string / hanger
        for sy in 0..(disco_y.saturating_sub(radius_y as usize)) {
            let idx = sy * cols + disco_x;
            grid[idx].ch = '│';
            grid[idx].fg = (100, 100, 100);
        }
    }

    // 5. Render Bouncing Equalizer Bars (bottom)
    let audio_vol = disco.audio_visualizer.get_volume();
    let max_bar_h = (rows / 5).clamp(4, 10);
    let bar_spacing = 3;
    let mut rng_eq = LcgRng::new(999);
    for x in (0..cols).step_by(bar_spacing) {
        let val = if audio_vol > 0.005 {
            let freq1 = 2.4;
            let freq2 = 5.6;
            let phase = x as f32 * 0.15;
            let sin_v1 = (disco.time_elapsed * freq1 + phase).sin().abs();
            let sin_v2 = (disco.time_elapsed * freq2 - phase * 0.5).cos().abs() * 0.4;
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
    for c in &disco.confetti {
        if c.x >= 0.0 && c.y >= 0.0 {
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
    }

    // library 4.1: render the centered system logo from the live OS info
    let logo_text = get_system_info().logo_text;
    let lines = render_logo_block(&logo_text, None);
    let logo_h = lines.len();
    let logo_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let logo_x = cols.saturating_sub(logo_w) / 2;
    let logo_y = rows.saturating_sub(logo_h) / 2;

    if logo_x < cols && logo_y < rows {
        let r = ((disco.time_elapsed * 4.5).sin() * 127.0 + 128.0) as u8;
        let g = ((disco.time_elapsed * 4.5 + 2.0).sin() * 127.0 + 128.0) as u8;
        let b = ((disco.time_elapsed * 4.5 + 4.0).sin() * 127.0 + 128.0) as u8;
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
