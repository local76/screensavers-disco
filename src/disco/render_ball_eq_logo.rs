use crate::runner::core::TerminalCell;
use crate::runner::core::LcgRng;
use crate::runner::toolkit::sys_info::get_system_info;
use crate::runner::core::logo_block::render_logo_block;
use crate::disco::types::NEON_COLORS;
use crate::disco::Disco;

pub fn render_disco_ball(disco: &Disco, grid: &mut [TerminalCell], cols: usize, rows: usize, disco_x: usize, disco_y: usize) {
    if disco.disco_ball_opt != 1 {
        return;
    }

    let num_rays = 8;
    let base_angle = disco.time_elapsed * 0.7;
    let max_ray_len = 35.0;

    // Render Disco Ball light rays sweeping
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

    // Render Shimmering Disco Ball itself
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

pub fn render_equalizer(disco: &Disco, grid: &mut [TerminalCell], cols: usize, rows: usize) {
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
}

pub fn render_logo(disco: &Disco, grid: &mut [TerminalCell], cols: usize, rows: usize) {
    use std::sync::OnceLock;
    static LOGO_TEXT: OnceLock<String> = OnceLock::new();
    let logo_text = LOGO_TEXT.get_or_init(|| {
        if disco.sys_refresh_timer < -500.0 {
            "TEST".to_string()
        } else {
            get_system_info().logo_text
        }
    });
    let lines = render_logo_block(logo_text, None);
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
