use crate::runner::core::TerminalCell;
use crate::disco::Disco;

pub fn render_stars(disco: &Disco, grid: &mut [TerminalCell], cols: usize, rows: usize) {
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
}
