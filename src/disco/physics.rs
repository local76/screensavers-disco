use crate::runner::core::TerminalCell;
use crate::disco::Disco;
use crate::disco::render_stars;
use crate::disco::render_ball_eq_logo;

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

    // 2. Render background stars
    render_stars::render_stars(disco, grid, cols, rows);

    // 3. Render Disco Ball light rays sweeping & Shimmering Disco Ball
    render_ball_eq_logo::render_disco_ball(disco, grid, cols, rows, disco_x, disco_y);

    // 5. Render Bouncing Equalizer Bars (bottom)
    render_ball_eq_logo::render_equalizer(disco, grid, cols, rows);

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
    render_ball_eq_logo::render_logo(disco, grid, cols, rows);
}
